# Fleetform Cloud — Product Requirements Document v1.0

**Status:** Draft for build sequencing. Not production.
**Audience:** Founder + whoever implements the next phase.
**Date:** 2026-09-19

---

## 1. One-sentence product

Fleetform is the easy button for cloud infrastructure: describe what you need in a box (or a small config file), preview it, test it in Docker, then apply it for real on AWS, GCP, Hetzner, Cloudflare, Railway, or Supabase — from a UI or a CLI — with secrets kept in an encrypted vault.

Terraform/OpenTofu stay the power tools. Fleetform is the product people finish in minutes.

---

## 2. What we are not (yet)

We are **not in production**. Phase 1 on `engine/phase-1-live-apply` can launch one EC2 when `FLEETFORM_LIVE=1`. The pretty SaaS UI does not apply cloud state. This document is the plan to close that gap in order, with a test gate on every phase.

Do not start Phase N+1 until Phase N exit criteria are green.

---

## 3. Who it is for

| Persona | Pain | Job to be done |
|---|---|---|
| Solo founder / indie hacker | TF is a second job | “Give me a web app stack on the cheapest honest provider” |
| Nigerian / African startup | AWS console + IAM is hostile | “One box, one vault, Paystack billing, no surprise bill” |
| Backend engineer | Repeatable env across laptop and cloud | “Same project: docker sandbox tonight, live tomorrow” |
| Agency / small team | Many client accounts | Multi-tenant orgs, roles, audit log |
| Enterprise platform owner | Policy + SSO | SSO, private runners, spend caps, SCIM later |

Primary beachhead: **solo and 2–10 person teams** who already know they want a VM + DB + bucket, and hate writing 200 lines of HCL to get it.

---

## 4. Product principles

1. **UI and CLI are the same product.** Anything the UI can apply, the CLI can apply. Same engine, same state, same run id.
2. **Never lie about apply.** Status is `planned` | `applying` | `running` | `failed` | `destroyed`. “Applied” without a provider resource id is a bug.
3. **Preview before money.** Plan is free. Live apply is explicit. Destroy of live resources is explicit.
4. **One intent, many providers.** User picks outcome (“web app + postgres + object storage”), not raw resource types first.
5. **Secrets never in git or chat logs.** Vault is the only store the engine reads.
6. **Sandbox first.** Docker compose / local containers prove the shape. Live is a promotion, not a different product.
7. **Easy beats complete.** Ship 20% of Terraform surface that covers 80% of first-week stacks.

---

## 5. Core user journeys

### 5.1 Prompt to live (north star)

1. Sign in.
2. Create project / pick org.
3. Paste or type: “Next.js app, Postgres, S3-compatible bucket, HTTPS, region EU.”
4. Pick provider pack (AWS | GCP | Hetzner+Cloudflare | Railway | Supabase).
5. AI returns a **plan card**: resources, monthly estimate, blast radius.
6. User can edit the card (size, region, public/private).
7. **Test in Docker** — local/sandbox run, no cloud bill.
8. **Apply live** — engine writes cloud, waits for healthy, stores ids.
9. Dashboard shows running resources + logs + cost so far.
10. Destroy tears down in reverse order.

### 5.2 CLI twin

```
fleetform login
fleetform project new shop
fleetform intent "web + postgres + bucket" --provider hetzner
fleetform plan
fleetform sandbox up
fleetform apply --live
fleetform show
fleetform destroy --live
```

Same project id. Same state backend.

### 5.3 Power user

Bring your own `main.tf` / Fleetform HCL. Skip the prompt. Still get plan, sandbox, vault, UI.

---

## 6. Functional requirements

### 6.1 Control plane (SaaS)

- Email + password, then GitHub OAuth.
- Orgs, members (owner / admin / operator / viewer).
- Projects + environments (`sandbox`, `staging`, `prod`).
- Runs: queued job with logs streamed to UI.
- Audit log: who applied what, when, which ids.
- Billing via Paystack (NG) + Stripe (intl).
- Rate limits by plan (see `docs/PRICING.md`). Premium+ uncapped within fair use.

### 6.2 Engine

- Desired spec → graph → plan → apply → refresh → destroy.
- Resource kinds Phase 2+: instance/VM, network, firewall/SG, SSH key, object bucket, managed DB, serverless function, DNS.
- Idempotent by address (`aws_instance.web`).
- Wait for *ready*, not just API 200.
- Docker sandbox driver that maps the same spec to Compose services.

### 6.3 Providers (order of integration)

1. AWS (EC2, VPC/SG, key pair, S3, RDS later)
2. Hetzner Cloud (cheap VMs; pair with Cloudflare DNS)
3. Cloudflare (DNS, tunnel, R2)
4. GCP (Cloud Run + Cloud SQL — maps to existing founder skill)
5. Railway (PaaS apply via API)
6. Supabase (project + DB + storage via management API)

A provider ships only with: auth, plan, apply, refresh, destroy, and a fixture test.

### 6.4 AI planner

- Single input box.
- Output is a **structured spec** (JSON/HCL), never hidden side effects.
- User must confirm plan.
- Model is not allowed to apply. Engine applies.
- Token/cost budget per run on Free/Standard.

### 6.5 Vault

- Per-org encrypted secret store (AES-256-GCM, envelope key per org).
- Values never returned in list APIs; reveal is audited.
- Engine reads at apply time into memory, not disk.
- Bring-your-own KMS later (Enterprise).

### 6.6 UI

- Dark canvas, one accent, huge type, pill nav (Byteship-class, not a terminal skin).
- Home: projects.
- Project: prompt box, plan card, sandbox toggle, Apply, Destroy.
- Run drawer: live logs.
- Vault page.
- Settings: members, billing, provider connections.
- Mobile usable for status + approve, not for first-time graph editing.

---

## 7. Non-functional

- Apply path p99 plan < 5s for starter stacks; live apply bounded by provider.
- No secret in logs, analytics, or LLM traces.
- Multi-tenant isolation at org id on every query.
- Backups of control-plane DB daily; vault keys separate from DB dumps.
- Status page + error budget once paid users exist.

---

## 8. Explicit non-goals (12 months)

- Replace every Terraform provider.
- Kubernetes cluster lifecycle as v1.
- Multi-cloud *in one stack* (one provider pack per environment).
- Unattended apply from the model with no human confirm.

---

## 9. Success metrics

| Phase | Exit metric |
|---|---|
| 1 | Live EC2 reaches `running` and destroy removes it |
| 2 | UI Apply creates the same instance; state ids match |
| 3 | Starter pack (VM + SG + SSH + bucket) apply + destroy in one test account |
| 4 | Docker sandbox boots an equivalent stack locally |
| 5 | 10 design-partner projects complete prompt → plan → apply |
| 6 | Paid conversion ≥ 5% of weekly actives who ran a live apply |

---

## 10. Risks

- **Scope explosion.** Multi-cloud + AI + vault + billing in one sprint recreates this week’s hijack. Sequence wins.
- **Cost accidents.** Live apply without caps burns users and you. Hard spend cap on Free.
- **Secret leakage via the model.** Planner sees names of secrets, never values.
- **Provider API drift.** Each provider is a module with contract tests.
- **Trust.** One fake “applied” in the UI resets the brand.

---

## 11. Source of truth

| Artifact | Role |
|---|---|
| `main.tf` / intent spec | Desired |
| `.fleetform/state.json` or remote state | Actual |
| Run record in control plane | Who / when / logs |
| This PRD + `docs/ROADMAP.md` | What we build next |
