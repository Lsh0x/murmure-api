# Multi-stage build for Murmure gRPC Server
FROM rust:1.75 as builder

WORKDIR /app

# Install protobuf compiler
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency manifests
COPY src-tauri/Cargo.toml src-tauri/Cargo.lock ./
COPY proto ./proto

# Create a dummy source to build dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release --bin murmure-server && \
    rm -rf src

# Copy source code
COPY src-tauri/src ./src
COPY src-tauri/build.rs ./build.rs

# Build the application
RUN cargo build --release --bin murmure-server

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/murmure-server /usr/local/bin/murmure-server

# Copy resources (models and cc-rules) - these should be mounted or copied
# COPY resources /app/resources

# Set environment variables
ENV MURMURE_GRPC_PORT=50051
ENV MURMURE_LOG_LEVEL=info
ENV RUST_LOG=info

# Expose gRPC port
EXPOSE 50051

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD true || exit 1

# Run the server
CMD ["murmure-server"]

