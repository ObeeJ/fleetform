<div align="center">

# Fleetform

**A Rust-powered Infrastructure as Code CLI with a real-time web dashboard.**

[![CI](https://github.com/ObeeJ/fleetform/actions/workflows/checks.yml/badge.svg)](https://github.com/ObeeJ/fleetform/actions/workflows/checks.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

</div>

Fleetform is a Terraform/OpenTofu-style Infrastructure as Code tool. A Rust CLI core parses HCL, plans changes, and can apply an EC2 instance until AWS reports `running`. A Go Fiber dashboard reads the files the CLI wrote. The dashboard does not launch machines.

## Phase 1 product loop

```bash
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
export AWS_DEFAULT_REGION=us-east-1

cargo run -- init
cargo run -- validate
cargo run -- plan          # dry plan unless FLEETFORM_LIVE=1
FLEETFORM_LIVE=1 cargo run -- apply
cargo run -- show          # expect status=running and an i- id
cd fiber && go run main.go # http://localhost:3001 reads plan + state
FLEETFORM_LIVE=1 cargo run -- destroy
```

Without `FLEETFORM_LIVE=1`, apply records `planned` in `.fleetform/state.json` and does not call AWS. That is intentional.

The example `main.tf` uses a placeholder AMI. Live apply resolves current Amazon Linux 2023, or uses `FLEETFORM_AMI`.

Success means `.fleetform/state.json` contains `"status": "running"` and an instance id. Anything else is not launched.

## Features

- **HCL configuration** parsed from `main.tf`
- **Plan from desired vs state** — create / no-op based on managed resources
- **Live EC2 apply** gated by `FLEETFORM_LIVE` — waits until the instance is running
- **Destroy** terminates ids stored in state when live
- **Local state** in `.fleetform/state.json`, optional S3 upload when `FLEETFORM_S3_BUCKET` is set
- **Dashboard** shows plan and state files; it will not say a machine is running unless state says so

## Architecture

```
CLI (Rust)  -->  .fleetform/state.json + fleetform_plan.json  -->  Dashboard (Go Fiber)
   |
   +---- AWS EC2 (only when FLEETFORM_LIVE=1)
```

## Current maturity

Version `0.1.0`. Phase 1 is **one EC2 instance, default VPC, idempotent re-apply by address**.

Not yet: custom VPC/subnet/SG, S3 apply, provider plugins as the source of truth, or a SaaS Apply button that mutates AWS.

## License

Fleetform is released under the [MIT License](LICENSE).
