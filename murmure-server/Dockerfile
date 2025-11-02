# Multi-stage build for Murmure gRPC Server using cargo-chef
FROM rust:1.85 AS chef
WORKDIR /app
RUN cargo install cargo-chef --locked

# 1) Plan dependencies
FROM chef AS planner
WORKDIR /app
# Copy only files that affect dependency resolution (keep this minimal!)
COPY Cargo.toml Cargo.lock ./
COPY murmure-lib/Cargo.toml ./murmure-lib/
COPY murmure-server/Cargo.toml ./murmure-server/
COPY murmure-server/build.rs ./murmure-server/
# If build-deps read proto schema at build-time, include it here so cache invalidates when it changes
COPY proto ./proto
RUN cargo chef prepare --recipe-path recipe.json

# 2) Cook dependency layer (this compiles murmure-lib as a path dependency)
FROM chef AS cacher
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# 3) Build the application
FROM chef AS builder
WORKDIR /app
# Build-time tooling
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler \
 && rm -rf /var/lib/apt/lists/*

# Bring in the full workspace source
COPY . .
# Reuse the cooked dependency cache
COPY --from=cacher /app/target /app/target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
# Build binary
RUN cargo build --release --bin murmure-server

# 4) Runtime image
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    netcat-openbsd \
 && rm -rf /var/lib/apt/lists/*

# Copy the binary
COPY --from=builder /app/target/release/murmure-server /usr/local/bin/murmure-server

# Optionally copy resources (or mount them at runtime)
# COPY resources /app/resources

ENV MURMURE_GRPC_PORT=50051 \
    MURMURE_LOG_LEVEL=info \
    RUST_LOG=info

EXPOSE 50051
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
  CMD nc -z localhost 50051 || exit 1
CMD ["murmure-server"]
