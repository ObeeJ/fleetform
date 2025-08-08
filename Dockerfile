FROM rust:1.83-alpine AS builder
WORKDIR /app
COPY . .
RUN apk add --no-cache musl-dev gcc protobuf-dev && \
    cargo build --release

FROM scratch
COPY --from=builder /app/target/release/fleetform /fleetform
ENTRYPOINT ["/fleetform"]