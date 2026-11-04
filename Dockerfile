# Multi-stage Dockerfile for Autonomous Cloud Run Reddit Agent
# Stage 1: Build Rust core binary
FROM rust:1.80-slim-bullseye AS builder
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
# Cache dependencies with a dummy src/main.rs
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "pub fn lib() {}" > src/lib.rs && cargo build --release || true
COPY src ./src
RUN cargo build --release --bin reddit-agent

# Stage 2: Runtime image with Bun and CA certificates
FROM oven/bun:1.3-slim AS runner
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
WORKDIR /app

COPY --from=builder /usr/src/app/target/release/reddit-agent /usr/local/bin/reddit-agent
COPY package.json bun.lock ./
COPY scripts ./scripts
RUN bun install --production

ENV PORT=8080
ENV RUST_LOG=info
EXPOSE 8080

CMD ["bun", "run", "scripts/cloud_run_server.ts"]
