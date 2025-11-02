# Multi-stage build for Murmure gRPC Server
FROM rust:1.85 as builder

WORKDIR /app

# Install protobuf compiler
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency manifests and build script
# Create src-tauri structure to match expected paths in build.rs
RUN mkdir -p src-tauri
COPY src-tauri/Cargo.toml ./src-tauri/
COPY src-tauri/build.rs ./src-tauri/build.rs
COPY proto ./proto

WORKDIR /app/src-tauri

# Create a dummy source to build dependencies
# We need both src/lib.rs (for the library) and src/main.rs (for the binary)
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub mod server { pub mod grpc { pub mod murmure { pub mod transcription_service_server { pub struct TranscriptionServiceServer; } } } }" > src/lib.rs && \
    cargo build --release --bin murmure-server && \
    rm -rf src

# Copy source code
COPY src-tauri/src ./src

# Build the application
RUN cargo build --release --bin murmure-server

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    netcat-openbsd \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/src-tauri/target/release/murmure-server /usr/local/bin/murmure-server

# Copy resources (models and cc-rules) - these should be mounted or copied
# COPY resources /app/resources

# Set environment variables
ENV MURMURE_GRPC_PORT=50051
ENV MURMURE_LOG_LEVEL=info
ENV RUST_LOG=info

# Expose gRPC port
EXPOSE 50051

# Health check - test if server is listening on port
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD nc -z localhost 50051 || exit 1

# Run the server
CMD ["murmure-server"]
