# Build stage for Rust
FROM rust:1.77-slim-bullseye AS rust-builder

WORKDIR /app
COPY . .

# Install build dependencies
RUN apt-get update && apt-get upgrade -y && \
    apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Build the Rust application
RUN cargo build --release

# Build stage for Go
FROM golang:1.22-alpine3.20 AS go-builder

WORKDIR /app
COPY fiber/ ./fiber/
WORKDIR /app/fiber
RUN apk update && apk upgrade && go build -o fiber

# Final stage
FROM alpine:3.18.4

WORKDIR /app
RUN apk add --no-cache ca-certificates

# Copy the compiled binaries
COPY --from=rust-builder /app/target/release/fleetform /usr/local/bin/
COPY --from=go-builder /app/fiber/fiber /usr/local/bin/
COPY fiber/static /app/static

# Set environment variables
ENV RUST_LOG=info

# Expose port for the web interface
EXPOSE 3000

# Start the application
ENTRYPOINT ["fleetform"]
