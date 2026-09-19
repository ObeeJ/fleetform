<div align="center">

# Fleetform

**A Rust-powered Infrastructure as Code CLI with a real-time web dashboard.**

[![CI](https://github.com/ObeeJ/fleetform/actions/workflows/checks.yml/badge.svg)](https://github.com/ObeeJ/fleetform/actions/workflows/checks.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

</div>

Fleetform is a Terraform/OpenTofu-style Infrastructure as Code tool. A Rust CLI core handles configuration parsing, dependency resolution, planning, and applying changes, while a companion Go (Fiber) web server exposes a live dashboard for visualizing plans, diffs, and modules over WebSocket.

## Features

- **HCL-based configuration** — define providers, resources, and modules using familiar HCL syntax, parsed with `hcl-rs`
- **Dependency graph planning** — resource dependencies are resolved into a DAG (via `petgraph`) before execution
- **Real-time web dashboard** — a Go Fiber server serves plan data, state, module listings, and diffs, with live updates pushed over WebSocket
- **Workspaces** — create, select, list, and switch between isolated named workspaces
- **Module system** — fetch and cache reusable configuration modules from a registry
- **Pluggable state backends** — local file, AWS S3, and Consul, with file locking and automatic retries for safe concurrent access
- **OpenTofu provider protocol** — communicates with providers over the `tfplugin6` gRPC protocol (via `tonic`/`prost`)
- **Infrastructure testing** — run validation checks against your configuration before applying
- **Cross-platform** — builds and runs on Linux, macOS, and Windows

## Architecture

```
CLI (Rust)  <---->  Web Dashboard (Go Fiber)  <---->  State backends (file / S3 / Consul)
   |
   +---- tfplugin6 (gRPC) ----> providers (e.g. AWS)
```

- **Rust core** (`src/`) — CLI commands, HCL parsing, DAG planner, state management, provider protocol client.
- **Go Fiber server** (`fiber/`) — dashboard UI and JSON endpoints for plan/state/module/diff data, plus WebSocket updates.
- **State backends** (`src/state.rs`) — persist state to a local file, AWS S3, or Consul.

## Getting Started

### Prerequisites

- Rust stable toolchain
- Go 1.23+
- (Optional) Docker and Docker Compose
- (Optional) AWS credentials for the S3 backend, or a Consul agent

### Build and run locally

```bash
cargo build --release
cargo run -- init
cargo run -- plan
cargo run -- apply
cd fiber && go run main.go
# Dashboard: http://localhost:3001
```

### Production deploy (dashboard)

This is the path to run today on a single VM or any Docker host:

```bash
# Build and start the production web dashboard on port 3001
docker compose -f docker-compose.prod.yml up --build -d fleetform-web
```

The CLI is a command-line tool, not a long-running service. Build it as an image when you need to run commands in the same environment:

```bash
docker compose -f docker-compose.prod.yml --profile cli build fleetform-cli
docker compose -f docker-compose.prod.yml --profile cli run --rm fleetform-cli plan
```

After merge to `master`, GitHub Actions also builds images and (on push) publishes them to GHCR:

- `ghcr.io/obeej/fleetform-web:latest`
- `ghcr.io/obeej/fleetform-cli:latest`

## CLI Commands

| Command | Description |
|---|---|
| `fleetform init` | Initialize a new Fleetform configuration |
| `fleetform validate` | Validate configuration files |
| `fleetform hcl-validate` | Validate HCL syntax |
| `fleetform plan` | Create an execution plan |
| `fleetform apply` | Apply configuration changes |
| `fleetform destroy` | Destroy managed infrastructure |
| `fleetform fmt` | Format configuration files |
| `fleetform show` | Show current state |
| `fleetform config` | Show configuration |
| `fleetform providers` | List available providers |
| `fleetform state-mv` | Move resources within state |
| `fleetform test` | Run infrastructure tests |
| `fleetform workspace new\|select\|show\|list` | Manage workspaces |
| `fleetform module` | Manage reusable modules |
| `fleetform consul` | Read/write state to a Consul backend |
| `fleetform provision` | Provision resources |

## Web Dashboard

| Endpoint | Description |
|---|---|
| `GET /` | Interactive dashboard |
| `GET /ui` | Plan data |
| `GET /config` | Current configuration |
| `GET /state` | Current state |
| `GET /modules` | Module listing |
| `GET /diff` | Plan diff viewer |
| `WS /realtime` | Live WebSocket updates |

## Configuration

Copy `.env.example` to `.env`:

```bash
AWS_ACCESS_KEY_ID=your_access_key_here
AWS_SECRET_ACCESS_KEY=your_secret_key_here
AWS_DEFAULT_REGION=us-east-1
CONSUL_ENDPOINT=http://localhost:8500
```

## Current maturity

Fleetform is at version `0.1.0`. The CLI, dashboard, workspaces, and state backends exist. Some `plan` / `apply` paths still contain demo placeholders and are not a drop-in replacement for Terraform/OpenTofu in production cloud accounts. Treat today's production cut as: **dashboard + CLI containers ship and CI can build**.

## License

Fleetform is released under the [MIT License](LICENSE).
