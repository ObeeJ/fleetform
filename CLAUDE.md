# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project overview

Fleetform is an Infrastructure-as-Code CLI (positioned as a Rust-based alternative to OpenTofu/Terraform), made of two components in one repo:

- **Rust core** (`src/`) — the `fleetform` CLI: config parsing, dependency graph/planning, state management, and applying changes to AWS.
- **Go Fiber web server** (`fiber/`) — a real-time dashboard (WebSocket-based) that the Rust CLI spawns as a child process on every invocation to visualize plans/state/modules.

The codebase is early-stage: several commands (`plan`, `apply`, provider fetch helpers) contain hardcoded/demo logic (e.g. planning always references `aws_instance.example`/`aws_s3_bucket.my_bucket` rather than the parsed config) rather than fully wired end-to-end behavior. Check the actual command implementation in `src/commands/` before assuming Terraform-grade correctness.

## Build, lint, and test

### Rust CLI

```bash
cargo build --release        # release build
cargo run -- <command>       # e.g. `cargo run -- plan`
cargo test                   # run all tests
cargo test <test_name>       # run a single test, e.g. `cargo test test_run`
cargo fmt                    # format
cargo fmt --check            # CI formatting check
cargo clippy --all-targets --all-features -- -D warnings   # CI lint check
```

**Gotcha — default build target is pinned to Windows.** `.cargo/config.toml` sets `[build] target = "x86_64-pc-windows-msvc"` unconditionally, so a bare `cargo build`/`cargo run`/`cargo test` on Linux or macOS fails with `error[E0463]: can't find crate for 'core'` unless the msvc target is installed. On non-Windows, pass an explicit target:

```bash
cargo build --target x86_64-unknown-linux-gnu   # Linux
cargo build --target x86_64-apple-darwin        # macOS
```

(`.github/workflows/checks.yml` runs bare `cargo build --release` across ubuntu/windows/macos runners — be aware this same target-pinning issue applies there too.)

There are very few tests in the repo today (`src/commands/workspace_test.rs`, `src/ffi.rs`); most "test" commands (`fleetform test`, `WorkspaceTest`, `ConsulTest`) are informal manual-verification routines invoked via the CLI, not `#[test]` functions.

### Go Fiber dashboard

```bash
cd fiber && go run main.go     # serves http://localhost:3001
cd fiber && go build -o fiber  # build binary
```

### Docker

```bash
docker-compose up -d                 # fleetform-cli, fleetform-web (fiber), redis
scripts/docker-dev.sh {up|down|logs|cli|build}
```

## Architecture

**CLI entrypoint** (`src/main.rs`): a `clap` `Cli`/`Commands` enum dispatches to `src/commands/*.rs`. On every invocation, `start_ui_server()` spawns `go run main.go` in `fiber/` on a background thread before parsing args — so running the CLI at all requires a working Go toolchain and `fiber/` deps, even for commands that don't need the dashboard. `-C/--chdir` changes directory before executing; `FLEETFORM_CLI_ARGS` can inject extra CLI args (see `CONTRIBUTING.md`).

Two command wiring styles coexist:
- Most commands (`plan`, `apply`, `destroy`, `init`, `validate`, `show`, `config`, `providers`, `test`, `consul`, `provision`, `fmt`, `hcl_validate`, `state_mv`, `workspace_test`, `consul_test`) expose a plain `pub async fn run()`, matched directly in `main.rs`.
- `workspace` and `module` instead use a `Command` trait + `Meta { working_dir, streams }` pattern with clap subcommands (`src/commands/workspace.rs`, `src/commands/module.rs`) — follow this pattern if adding new multi-subcommand command groups.

**Config** (`src/config.rs`): loads `fleetform.{hcl,yaml,json}` from the current directory. HCL is parsed with a hand-rolled brace-counting parser in `src/hcl.rs` (`parse_blocks`/`parse_terraform_config`/`validate_hcl_syntax`) rather than the `hcl-rs` crate that's already a dependency in `Cargo.toml`. `parse_hcl_basic` in `config.rs` is an even cruder fallback that just checks lines for the substrings `"provider"`/`"resource"`.

**State** (`src/state.rs`): JSON state at `.fleetform/state.json`, guarded with `fs4` file locks and a 3-retry helper (`with_retry`) for both read and write. Also supports remote backends: `write_s3`/read via `aws-sdk-s3` (bucket from `FLEETFORM_S3_BUCKET` env var), and `write_consul`/`read_consul` via `reqwest` against a Consul KV endpoint.

**Dependency graph** (`src/dag.rs`): `ResourceGraph` wraps `petgraph::Graph`, with `add_resource`/`add_dependency`/`get_ordered_resources` (toposort) used by `plan`.

**Workspaces** (`src/workspace.rs`): `.fleetform/workspaces/<name>/`, current workspace name tracked in `.fleetform/current_workspace`. Note `Workspace::get_state_path()` actually resolves to `<working_dir>/fleetform-<name>.json`, not under `.fleetform/`.

**Modules** (`src/modules.rs`, `src/modules/{init,update}.rs`, `src/registry.rs`): `ModuleRegistry` fetches modules by URL into a local `modules/` cache dir; `fleetform module init` scaffolds `main.tf`/`variables.tf`/`outputs.tf` under `modules/<name>/`.

**Provisioning** (`src/provisioner.rs`): the one component that talks to real AWS — uses `aws-sdk-ec2`/`aws-sdk-s3` directly to create EC2 instances and S3 buckets, reading resource blocks out of a literal `main.tf` file via the `hcl::parse_blocks` parser (independent of whatever `fleetform.hcl` config was loaded).

**Provider registry** (`src/provider.rs`): `Source` struct and free functions for fetching provider metadata from `registry.opentofu.org`; `apply_changes`/`list_available_providers` are largely stub/demo implementations.

**OpenTofu provider protocol scaffolding** (`proto/tfplugin6.proto`, `src/tofu/{plugin,provider,schema}.rs`, `build.rs`): sets up `tonic`/`prost` codegen for the tfplugin6 gRPC protocol real Terraform/OpenTofu providers speak. `build.rs` falls back to writing a placeholder `tfplugin6.rs` if `protoc` isn't available on `PATH` — a vendored Windows `protoc.exe`/`protoc-28.2-win64.zip` is checked into `bin/`/`proto/` for that platform only.

**Go Fiber dashboard** (`fiber/`): `main.go` wires routes to `fiber/handlers/*.go` — `/ui`, `/config`, `/state`, `/modules`, `/diff`, and a `/realtime` WebSocket. Static assets served from `fiber/static/`. Handlers read files the Rust CLI writes to disk (e.g. `plan` writes `fleetform_plan.json` to the repo root, which the dashboard/`/ui` endpoint consumes) — the two sides communicate via the filesystem and simple HTTP calls (`plan.rs` also does a `GET http://localhost:3001/ui` after writing the plan), not a shared API contract.

## Known repo quirks

- `src/provisioner.rs.fixed` and `src/provisioner.rs.new` are stray duplicate files, not referenced by any `mod` declaration in `src/main.rs` and therefore never compiled. Don't confuse them with the real `src/provisioner.rs`.
- `Cargo.toml` still has placeholder metadata (`authors = ["Your Name <your.email@example.com>"]`, `description = "A Rust-based OpenTofu provider implementation"`) inconsistent with the broader "full IaC CLI" framing in `README.md`.
- `test.tf` in the repo root is UTF-16-encoded (not plain UTF-8 like other `.tf` files).

## Configuration

Copy `.env.example` to `.env` and set AWS credentials plus `CONSUL_ENDPOINT` for provisioning/remote-state commands:

```
AWS_ACCESS_KEY_ID=...
AWS_SECRET_ACCESS_KEY=...
AWS_DEFAULT_REGION=us-east-1
CONSUL_ENDPOINT=http://localhost:8500
```

Infrastructure config for the CLI itself goes in `fleetform.hcl` (or `.yaml`/`.json`) in the working directory — see `examples/main.tf` and `fleetform.hcl` for the expected `provider`/`resource` block shape.
