# Fleetform — Roadmap

Rule: **test the phase, then move.** No parallel product surfaces.

Engine branch for Phase 1 code: `engine/phase-1-live-apply` (PR #7).
This docs branch does not replace that engine work.

---

## Phase 0 — Stop the lies (done / merge now)

- Plan from `main.tf` vs state
- Dry apply vs `FLEETFORM_LIVE=1`
- Wait until instance `running`
- Dashboard reads files; no fake plan rows

**Exit:** `show` prints `status=running` and an `i-` id after live apply. Destroy with live flag terminates it.

---

## Phase 1 — Honest AWS VM (current build)

- Custom VPC + subnet + security group + SSH key pair
- Real S3 bucket create/delete (unique name, no `my-bucket`)
- Plan destroy when a resource is removed from config
- Refuse destroy-without-live if state has cloud ids
- CLI + file dashboard only

**Tests:** unit plan graph; integration against an isolated AWS account or LocalStack for S3 + mocked EC2; manual live apply on a throwaway account once.

**Exit:** one command creates reachable SSH VM + private bucket; destroy leaves zero billed resources tagged `ManagedBy=fleetform`.

---

## Phase 2 — UI Apply is the same engine

- Control plane: login, one org, one project
- UI buttons Plan / Apply / Destroy enqueue a run
- Runner execs the Rust engine (or a thin Go wrapper) against project files
- UI shows run logs + managed ids from state
- Rate limit Free: 20 plans / day, 3 live applies / day

**Tests:** API test “Apply without live flag → planned only”; “Apply live → run record contains instance id”.

**Exit:** clicking Apply on the website creates the same resource the CLI would. No second source of truth.

---

## Phase 3 — Multi-tenant + vault + billing stub

- Orgs, roles, invite
- Encrypted vault (create / list names / delete / inject on apply)
- Paystack + Stripe checkout for Standard / Premium
- Spend cap env per project
- Audit log

**Tests:** tenant A cannot read tenant B state; vault value never appears in run logs.

**Exit:** a second user can join an org and apply using a stored AWS key without pasting it into chat.

---

## Phase 4 — Docker sandbox

- `fleetform sandbox up` generates Compose from the same spec
- UI toggle “Test locally”
- Promote sandbox → live keeps names and intent, swaps driver

**Tests:** sandbox boots nginx + postgres + minio for the starter pack; teardown is clean.

**Exit:** a user can develop tonight without AWS and promote tomorrow.

---

## Phase 5 — Provider packs (one at a time)

Order: Hetzner → Cloudflare DNS/R2 → GCP Cloud Run → Railway → Supabase.

Each pack: auth, plan, apply, refresh, destroy, cost estimate, fixture test.

**Exit for a pack:** design partner applies and destroys twice without manual console cleanup.

---

## Phase 6 — AI intent box

- Single textarea → structured spec → plan card
- Confirm required
- Model never receives vault values
- Token budget by plan

**Tests:** golden prompts (“static site”, “api + db”, “bucket only”) produce valid specs; jailbreak prompts cannot dump secrets.

**Exit:** 10 strangers get a working preview from one sentence without writing HCL.

---

## Phase 7 — Easy mode polish + Enterprise

- Templates: static site, API+DB, worker+queue
- Cost forecast and idle-resource nag
- SSO / SAML, private runner, invoice, SLA
- Status page

**Exit:** Premium NPS / qualitative “easier than TF” from 20 interviews; Enterprise has one paying design partner or we do not build SSO yet.

---

## What we will not do in Q4 2026

- Six providers in parallel
- Kubernetes as a starter pack
- Auto-apply from the model
- Shipping PR #6 Apply as if it were Phase 2
