# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Read this first

**`FLEETFORM_LIVE=1` creates real, billed AWS resources.** Without it, `apply` only records what it *would* do into local state and touches nothing in AWS. Never set it to "test" something, and never suggest a user set it without saying plainly that it will cost money and that `destroy` is the only way back.

**`docs/ROADMAP.md` governs sequencing, and it binds AI sessions specifically.** Its rule is "test the phase, then move — no parallel product surfaces," and `docs/BUSINESS.md` adds: *"An AI session does not open a new surface without an exit-criteria check on the current phase."* Read `docs/ROADMAP.md` and `docs/PRD.md` before starting work that isn't a bug fix. The founder owns sequence and can overrule, but the default is to finish the current phase.

**The PRD's second principle is "never lie about apply."** A status of `applied` without a provider resource id is a bug. Applies to output, state, and the dashboard alike.

## Project overview

Fleetform is an Infrastructure-as-Code CLI — a simpler Terraform/OpenTofu alternative aimed at solo founders and small teams. Two components in one repo:

- **Rust core** (`src/`) — the `fleetform` CLI: HCL parsing, planning, state, and AWS calls.
- **Go Fiber server** (`fiber/`) — a dashboard that *reads the files the Rust side writes*. It is a viewer, not a second engine.

Maturity: Phase 1 of `docs/ROADMAP.md`, and not yet at its exit criteria. The EC2 apply path is genuinely wired to AWS, but on `master` a parser bug means `plan` finds no resources in any real `.tf` file (see quirks), there is no S3 bucket creation (explicitly skipped in `apply_plan`), no custom VPC/subnet, and removing a resource from config does not plan a destroy. **Verify against the code before believing any claim of completeness — including this file's.**

## Build, lint, and test

```bash
cargo build --release
cargo run -- <command>        # e.g. cargo run -- plan
cargo test                    # 6 test fns across engine.rs, ffi.rs, workspace_test.rs
cargo test <name>             # single test
cargo fmt --check             # CI gate, no continue-on-error
cargo clippy --all-targets --all-features -- -D warnings   # CI runs this with continue-on-error
```

`cargo build` works on every host — `.cargo/config.toml` no longer pins a default target. To cross-compile for Windows: `cargo build --release --target x86_64-pc-windows-msvc`.

`build.rs` compiles `proto/tfplugin6.proto` and **silently falls back to a placeholder** if `protoc` is missing, so a local build without protobuf installed differs from CI's.

Note `cargo test` recompiles the whole tree in the test profile, separate from `--release`. With the aws-sdk dependency tree that is slow (tens of minutes cold).

### Go dashboard

```bash
cd fiber && go run main.go     # http://localhost:3001
```

### Docker

```bash
docker compose up -d
docker compose -f docker-compose.prod.yml up --build -d fleetform-web
```

## Architecture

**Entrypoint** (`src/main.rs`): `clap` dispatches to `src/commands/*.rs`. The Go dashboard is spawned **only when `FLEETFORM_UI=1`** — the CLI does not require a Go toolchain otherwise. `-C/--chdir` changes directory first.

Two command styles coexist: most expose a plain `pub async fn run()`; `workspace` and `module` use a `Command` trait + `Meta { working_dir, streams }` with clap subcommands. Follow the latter for new multi-subcommand groups.

**Engine** (`src/engine.rs`) — *the important file.* Owns the whole lifecycle:
- `load_desired()` reads **`main.tf`** (not `fleetform.hcl` — see quirks) and parses it
- `build_plan()` diffs desired blocks against `state.managed` by address (`aws_instance.example`). It is **pure and offline** — it never calls AWS, so it cannot detect drift
- `apply_plan()` / `apply_instance()` create resources, gated on `plan.live`
- `destroy_all()` terminates recorded instances and waits for `terminated`
- Instances are tagged `ManagedBy=fleetform`, `Name`, `fleetform:address`
- `client_token()` gives RunInstances a stable idempotency token per address

Instances currently launch with **no key pair and no security group**, into the default VPC — so "reachable SSH VM" (the Phase 1 exit criterion) is not achievable on `master` today.

**State** (`src/state.rs`): JSON at `.fleetform/state.json`. `ManagedResource { address, resource_type, name, id, status, public_ip }` — `id` is the cloud id and its presence is what distinguishes "really exists" from "only planned". Status vocabulary follows AWS: `planned`, `pending`, `running`, `available`, `skipped`.

`write_s3`/`write_consul` exist but are **write-only sinks** — `load()` only ever reads the local file, so these are not functioning remote backends yet. `read_consul` returns an *empty state* on connection failure, which would make a plan propose recreating everything; it is currently only reachable from the manual `consul` command.

**HCL** (`src/hcl.rs`): hand-rolled brace-counting parser, *not* the `hcl-rs` crate that is already in `Cargo.toml` but unreferenced. It is fragile — see quirks.

**Workspaces** (`src/workspace.rs`): `.fleetform/workspaces/<name>/`, current name in `.fleetform/current_workspace`. `get_state_path()` resolves to `<working_dir>/fleetform-<name>.json`, which is *not* where `state::load()`/`save()` read and write — they are hardcoded to `.fleetform/state.json`.

**Dashboard** (`fiber/`): routes in `main.go` → `fiber/handlers/*.go` (`/ui`, `/config`, `/state`, `/modules`, `/diff`, `/realtime` WebSocket). It reads `fleetform_plan.json` and `.fleetform/state.json` from disk. The contract between Rust and Go is the filesystem — there is no API, and Go must never write state.

**Dead or stub code — do not build on it:** `src/dag.rs`, `src/provisioner.rs` (and its `.fixed`/`.new` copies), most of `src/provider.rs`, `config.rs::parse_hcl_basic`, and `commands/state_mv.rs` are orphaned by `engine.rs` or are stubs. `provider::list_available_providers()` returns hardcoded fake versions. This is why `cargo clippy` reports dead-code errors.

## Known quirks and traps

Several of these have fixes sitting in open PRs. Check `git log` and open PRs before spending time re-fixing one.

- **`plan` reports "0 to add" for any real `.tf` file, including this repo's own `main.tf`.** `parse_blocks` drops whitespace while accumulating a block *header*, so `resource "aws_instance" "example"` collapses to one token, `labels` comes back empty, and `address_of()` skips every block — while `validate` still says "Configuration is valid". This makes the whole engine a no-op on `master`.
- **`destroy` without `FLEETFORM_LIVE` wipes local state while the resources keep running.** It clears `state.managed` and returns `Ok`, discarding the ids needed to ever find those instances again. Do not run it against real state.
- **A failed apply orphans billed resources.** State is saved only after the whole apply succeeds, and the instance id is recorded only after `wait_running` returns — so a timeout between `RunInstances` and that point leaves a live instance with nothing in state naming it.
- **The parser panics on ordinary HCL.** `tags = { ... }` causes a subtract-overflow in `parse_block`. `//` comments are parsed as blocks and swallow the following resource. `#` inside a string truncates the line. Migrating to `hcl-rs` is the intended fix.
- **`build_plan` counts no-ops as changes**, so a converged stack reports "N to change" and `is_empty()` never returns true.
- **`main.tf` vs `fleetform.hcl`.** The engine reads `main.tf`. `src/config.rs` reads `fleetform.{hcl,yaml,json}` and is largely vestigial. Don't assume config.rs is on the apply path.
- **Region is not in state.** Apply in one region, then `destroy` with a different `AWS_REGION`, and the ids won't be found — the resources keep billing.
- **Attributes are never diffed.** Changing `instance_type` on a running instance yields "No changes".
- **SIGINT calls `process::exit(130)` immediately**, so Ctrl-C cannot flush state.
- `Cargo.lock` is committed; the root `Dockerfile` does not copy it, so the production image re-resolves dependencies.
- `test.tf` is UTF-16 and will not parse. `fleetform_plan.json`, `bin/protoc.exe`, `proto/*.zip` and `fiber/fiber.exe` are committed build artifacts.
- The dashboard on `saas/control-plane` shows pricing tiers that contradict `docs/PRICING.md` — different tiers *and* a different billing model.

## Environment

```
AWS_ACCESS_KEY_ID / AWS_SECRET_ACCESS_KEY / AWS_DEFAULT_REGION
CONSUL_ENDPOINT          # manual consul command only
FLEETFORM_LIVE=1         # perform real AWS mutations — costs money
FLEETFORM_UI=1           # also start the Go dashboard
FLEETFORM_WORKSPACE      # workspace name, default "default"
FLEETFORM_AMI            # override AMI; otherwise the latest Amazon Linux 2023 is resolved
FLEETFORM_S3_BUCKET      # upload state to S3 after apply
```

`.fleetform/` holds local state and is gitignored — keep it that way, especially as credentials and key material start living there.
