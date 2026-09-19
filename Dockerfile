# Production image for the Fleetform CLI.
# Uses current stable Rust so floating crates that require edition 2024 compile.
FROM rust:1.94.1-bookworm AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
        protobuf-compiler \
        libprotobuf-dev \
        pkg-config \
        libssl-dev \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Cargo.lock has to come along: without it the image re-resolves dependencies
# at build time, so the released binary is not the one CI tested, and a new
# upstream release can break or change it without a commit.
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./
COPY proto ./proto
COPY src ./src
COPY .cargo ./.cargo

RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --uid 10001 fleetform

COPY --from=builder /app/target/release/fleetform /usr/local/bin/fleetform
USER fleetform
ENTRYPOINT ["fleetform"]
