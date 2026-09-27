# Fleetform pricing

Currency: **USD list**, charge **NGN via Paystack** at spot + 5% FX buffer for NG customers.
Cloud provider bills are always the customer’s, never ours.

## Tiers

| | Free | Standard | Premium | Enterprise |
|---|---|---|---|---|
| Price | $0 | $19 / user / mo | $49 / user / mo | $399 / mo seatless up to 25 users, then quote |
| Annual | — | $190 (2 months free) | $490 | custom |
| Projects | 1 | 5 | Unlimited | Unlimited |
| Members | 1 | 5 | 20 | SSO + SCIM |
| Plans / day | 20 | 200 | Unlimited* | Unlimited* |
| Live applies / day | 3 | 30 | Unlimited* | Unlimited* |
| AI intent prompts / day | 5 | 50 | Unlimited* | Unlimited* + own key |
| Rate limits | Yes | Yes | Fair-use only (no hard cap) | Fair-use + contract |
| Docker sandbox | Yes | Yes | Yes | Yes + private runner |
| Vault secrets | 5 | 50 | 500 | Unlimited + customer KMS |
| Providers | AWS only | AWS + Hetzner | All shipped packs | All + private |
| Support | Docs | Email 48h | Chat 8h WAT | Slack + named owner |
| Audit log retention | 7 days | 30 days | 1 year | 3 years |
| SLA | None | None | 99.5% control plane | 99.9% + MSA |

\*Unlimited = fair use. Abuse (bot apply loops, crypto mining via our runner) is paused, not silently billed.

## Rate limits (implementation)

- Free / Standard: token bucket per org on `plan`, `apply`, `intent`.
- Premium / Enterprise: no product cap; still per-IP flood protection (DDoS), not a monetization limit.
- Live apply always requires confirm + project spend cap (default $25 on Free).

## Unit economics (targets)

| Cost | Note |
|---|---|
| Control plane / active org | Aim <$2/mo (SQLite	o Postgres + object logs) |
| AI prompt | Pass through + 30% on Standard; included pool on Premium |
| Support | Standard email only until 200 paying |
| Gross margin target | ≥ 75% on software; never resell raw AWS |

If AI costs blow the Free tier, cut Free prompts to 5 before raising Standard.

## Packaging rules

- Free exists to complete **one** real project (learn + trust).
- Standard is the default sell (“team of 2–5 shipping weekly”).
- Premium is for agencies and people who live in the intent box.
- Enterprise is not built until one written LOI. Quote ₦ / $ separately.

## Discounts

- NG / Africa startups: Standard at $12 for first 12 months (manual coupon).
- Students / OSS maintainers: Free + extra project, no live-apply raise.
- Annual prepaid only on Standard/Premium.
