# Multi-stage Dockerfile for Jard

# 1. Planner Stage: Prepare dependencies list for caching
FROM lukemathwalker/cargo-chef:latest-rust-1.75 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-json recipe.json

# 2. Builder Stage: Compiled the project
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching layer
RUN cargo chef cook --release --recipe-json recipe.json
# Build application
COPY . .
RUN cargo build --release

# 3. Runtime Stage: Minimal production image
FROM debian:bookworm-slim AS runtime
WORKDIR /app
# Install necessary SSL certificates and libraries for outbound calls
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/jard /usr/local/bin/jard

# Set operational environment variables
ENV RUST_LOG=info
ENV APP_ENVIRONMENT=production

# Health check
HEALTHCHECK --interval=30s --timeout=3s \
  CMD curl -f http://localhost:8080/health || exit 1

ENTRYPOINT ["/usr/local/bin/jard"]
