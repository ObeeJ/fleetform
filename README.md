<div align="center">

# Fleetform

**A Rust-powered Infrastructure as Code CLI with a real-time web dashboard.**

[![CI](https://github.com/ObeeJ/fleetform/actions/workflows/checks.yml/badge.svg)](https://github.com/ObeeJ/fleetform/actions/workflows/checks.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

</div>

Fleetform is a Terraform/OpenTofu-style Infrastructure as Code tool. A Rust CLI core handles configuration parsing, dependency resolution, planning, and applying changes, while a companion Go (Fiber) web server exposes a live dashboard for visualizing plans, diffs, and modules over WebSocket.

## Features

- **HCL-based configuration** — define providers, resources, and modules using familiar HCL syntax, parsed with `hcl-rs`
- **Dependency graph engine** — resources are modeled as a DAG (via `petgraph`) and topologically ordered; wiring the planner to build this graph from parsed configuration (rather than the current fixed example graph) is in progress
- **Real-time web dashboard** — a Go Fiber server serves plan data, state, module listings, and diffs, with live updates pushed over WebSocket
- **Workspaces** — create, select, list, and switch between isolated named workspaces
- **Module system** — fetch and cache reusable configuration modules from a registry
- **Pluggable state backends** — local file, AWS S3, and Consul, with file locking and automatic retries for safe concurrent access
- **OpenTofu provider protocol (in progress)** — vendors the `tfplugin6` protobuf/gRPC definitions (via `tonic`/`prost`) as the basis for provider communication; the provider client is currently a placeholder and does not yet perform real provider RPCs
- **Infrastructure testing** — run validation checks against your configuration before applying
- **Cross-platform** — builds and runs on Linux, macOS, and Windows

## Architecture

```
┌───────────────────────┐        ┌─────────────────────────┐        ┌───────────────────────────┐
│      CLI (Rust)        │        │   Web Dashboard (Go)     │        │   State Backends           │
│  • clap-based commands │ <----> │  • Fiber HTTP + WebSocket│ <----> │  • Local file (fs4 locking) │
│  • HCL config parsing  │        │  • Plan / diff / module  │        │  • AWS S3                   │
│  • DAG dependency graph│        │    views                 │        │  • Consul                   │
│  • Execution planning  │        │  • Real-time updates     │        │                             │
└───────────────────────┘        └─────────────────────────┘        └───────────────────────────┘
             │                                                                     │
             └───────────────────────── tfplugin6 (gRPC) ─────────────────────────┘
                                              │
                                   ┌─────────────────────┐
                                   │  Infra Providers     │
                                   │  (e.g. AWS)           │
                                   └─────────────────────┘
```

- **Rust core** (`src/`) — implements CLI commands, HCL config parsing, the dependency graph/planner, state management, and the OpenTofu provider protocol client.
- **Go Fiber server** (`fiber/`) — serves the dashboard UI and JSON endpoints for plan/state/module/diff data, and streams live updates over a WebSocket connection. It is launched automatically by the CLI on startup.
- **State backends** (`src/state.rs`) — persist infrastructure state to a local file, AWS S3, or Consul, with atomic writes, backup cleanup, and retry logic.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- [Go](https://go.dev/dl/) 1.24.4+ (see `fiber/go.mod`)
- (Optional) Docker & Docker Compose for containerized development
- (Optional) AWS credentials for the S3 backend, or a running Consul agent for the Consul backend

### Build & Run

```bash
# Build the CLI
cargo build --release

# Initialize a new Fleetform workspace
cargo run -- init

# Create an execution plan from your configuration
cargo run -- plan

# Apply the planned changes
cargo run -- apply

# Destroy managed infrastructure
cargo run -- destroy
```

The CLI automatically starts the Fiber web server (`fiber/`) on launch. Visit **http://localhost:3001** to view the dashboard.

### Running with Docker Compose

```bash
docker-compose up --build
```

This starts the Rust CLI container, the Go Fiber web dashboard (port `3001`), and a Redis instance used for caching.

> **Note:** `Dockerfile.cli` and `fiber/Dockerfile` expect `Cargo.lock` and `fiber/go.sum` respectively, but both files are currently gitignored and not committed. Generate them locally before building (`cargo generate-lockfile` and `cd fiber && go mod tidy`), or the Docker build will fail on a clean checkout.

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

Global flags: `-C, --chdir <DIR>` to run from a different working directory.

Run `fleetform --help` or `fleetform <command> --help` for full usage details.

## Web Dashboard

Once running, the Fiber server exposes:

| Endpoint | Description |
|---|---|
| `GET /` | Interactive dashboard (static UI) |
| `GET /ui` | Plan data |
| `GET /config` | Current configuration |
| `GET /state` | Current state |
| `GET /modules` | Module listing |
| `GET /diff` | Plan diff viewer |
| `WS /realtime` | Live WebSocket updates |

## Configuration

Copy `.env.example` to `.env` and fill in the values you need:

```bash
AWS_ACCESS_KEY_ID=your_access_key_here
AWS_SECRET_ACCESS_KEY=your_secret_key_here
AWS_DEFAULT_REGION=us-east-1
CONSUL_ENDPOINT=http://localhost:8500
```

AWS credentials are required for the S3 state backend and AWS resource provisioning.

`CONSUL_ENDPOINT` in `.env.example` is illustrative only — the `consul` command does not currently read it from the environment. Pass the endpoint and operation as positional arguments instead:

```bash
fleetform consul <endpoint> <read|write>
```

## Modules

Reusable resource configurations live under `modules/`, for example:

- `modules/aws-vpc` — AWS VPC module
- `modules/aws-ec2` — AWS EC2 instance module

See `examples/main.tf` for a sample configuration referencing AWS resources.

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for local development setup, Windows-specific build notes, code style, and testing guidelines.

```bash
cargo build
cargo test
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

CI runs on Ubuntu, Windows, and macOS via GitHub Actions (see `.github/workflows/checks.yml`).

## License

Fleetform is released under the [MIT License](LICENSE).
