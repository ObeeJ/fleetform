# Contributing to Fleetform

## Development Setup

### Rust Components
```bash
cargo build
cargo test
cargo run -- init
cargo run -- --help
```

### Environment Variables
```bash
# Set CLI arguments via environment
export FLEETFORM_CLI_ARGS="-C /path/to/project"
cargo run -- plan
```

## Windows Development

### File Locking Workarounds

**Windows Defender Exclusion** (Recommended):
1. Open Windows Security → Virus & Threat Protection
2. Go to Manage Settings → Exclusions → Add or remove exclusions
3. Add your fleetform project folder as an exclusion

**Build Issues**:
- If `cargo build --release` fails with file locking errors, use `cargo clean && cargo build`
- Retry operations are built into state management (3 attempts with 1-second delays)
- Restart your system if persistent locking issues occur

**CI/CD**: GitHub Actions includes 10-minute timeouts for Windows builds

## Architecture

- **Rust**: Complete CLI tool with async HTTP client (reqwest)
- **Configuration**: Supports HCL, JSON, YAML via serde
- **State Management**: JSON-based with fs4 file locking, retries, and cleanup
- **Providers**: HTTP-based provider registry integration

## Code Style

- Run `cargo fmt` for code formatting
- Run `cargo clippy` for linting
- Follow conventional commits
- Add documentation for public APIs

## Testing

- Add unit tests for new features
- Test CLI commands with `cargo run --`
- Ensure all CI checks pass
- Test cross-platform compatibility
- On Windows: Test with antivirus exclusions

## Features

- **CLI**: Full argument parsing with clap
- **Signal Handling**: Graceful shutdown on SIGINT (Unix only)
- **Directory Changes**: `-C` flag support
- **Environment Args**: `FLEETFORM_CLI_ARGS` support
- **Version Info**: `-v/--version` flags
- **File Locking**: Cross-platform with retry logic
- **State Management**: Atomic operations with backup cleanup