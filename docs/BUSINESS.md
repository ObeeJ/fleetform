# Fleetform — business, finance, marketing

## Positioning

**Category:** AI-assisted infrastructure console.
**Against Terraform:** not more HCL. Less HCL.
**Against Railway/Render:** you can still own AWS/Hetzner when you outgrow PaaS.
**Against ChatGPT + console:** we store state, ids, destroy, and secrets.

Promise: *Describe it. Preview it. Sandbox it. Apply it. Destroy it.*

## Why this can work from Nigeria

- Founder already runs production Go + Terraform-on-GCP + AWS mentally.
- Beachhead users are African/EU indie teams who find AWS IAM a tax.
- Paystack first means NG cards work. Stripe when the first EU user asks.
- Hetzner + Cloudflare is the honest cheap pack; AWS is the resume pack.

## Company setup (practical)

1. Keep building as a personal project until first $1k MRR.
2. Then: register a legal entity (NG Ltd is fine to start; consider Estonian e-Residency later for EU invoices).
3. Separate bank / Paystack account from personal.
4. Cap personal AWS on a **throwaway account** with a $50 hard budget. Never demo on Dwelix prod keys.
5. Time budget: 6 focused hours/week. Dwelix remains the salary. Fleetform is the asset.
6. Do not hire until Premium support is drowning you.

## Finance rules

- Do not pre-buy six provider certifications.
- Do not run GPU inference yourself in year one. Use a hosted cheap model; Premium can BYO key.
- Price in USD, collect in NGN, keep a 2-month opex buffer before ads.
- Measure: weekly live-applies, weekly signups, Standard starts, destroy-success rate (trust).
- Kill vanity: stars and landing-page visits without a live apply do not count.

## Go-to-market (cheap)

**Month 0–1 (now):** ship Phase 1+2. Record a 90-second loom: prompt/plan not required yet — “I typed apply, AWS shows running, destroy leaves $0.” Post on X from the founder account as a builder log, not a startup launch.

**Month 2:** 10 design partners (friends who already pay Hetzner/AWS). White-glove. No ads.

**Month 3:** Public Standard. Template gallery. One comparison page: “Same stack in TF vs Fleetform.”

**Channels that fit this founder**
- X threads: one failure, one fix, one screenshot of real state.json. No “we launched SaaS” until UI Apply is real.
- LinkedIn: infrastructure-for-founders, not tool-bro.
- Communities: Nigerian Dev Twitter, GCP/AWS user groups, Hetzner threads.
- Content: “IAM in 6 clicks we hid from you” and “why your TF repo is 40 files for a blog.”

**Do not:** Product Hunt before Phase 2 exit. Ads before 20 organic live applies.

## Competitive note

Pulumi, Terraform, OpenTofu, Wing, SST, Brainboard, Env0, Spacelift exist. They optimize for platform teams. We optimize for **the first production weekend**. If we chase their enterprise checklist first, we lose the only wedge we have: ease.

## Decision rights

The founder owns sequence. An AI session does not open a new surface without an exit-criteria check on the current phase.
