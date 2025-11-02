# Multi-stage build for Murmure gRPC Server
FROM rust:1.85 as builder

WORKDIR /app

# Install protobuf compiler
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency manifests and build scripts
COPY murmure-lib/Cargo.toml ./murmure-lib/
COPY murmure-server/Cargo.toml ./murmure-server/
COPY murmure-server/build.rs ./murmure-server/
COPY proto ./proto

# Build library dependencies first (create dummy sources for dependency resolution)
WORKDIR /app/murmure-lib
RUN mkdir -p src/engine && \
    echo "mod audio;" > src/lib.rs && \
    echo "pub mod config;" >> src/lib.rs && \
    echo "pub mod dictionary;" >> src/lib.rs && \
    echo "mod engine;" >> src/lib.rs && \
    echo "pub mod model;" >> src/lib.rs && \
    echo "pub mod transcription;" >> src/lib.rs && \
    echo "pub use config::ServerConfig;" >> src/lib.rs && \
    echo "pub use dictionary::Dictionary;" >> src/lib.rs && \
    echo "pub use model::Model;" >> src/lib.rs && \
    echo "pub use transcription::TranscriptionService;" >> src/lib.rs && \
    echo "pub fn placeholder() {}" > src/audio.rs && \
    echo "use anyhow::Result;" > src/config.rs && \
    echo "#[derive(Clone)]" >> src/config.rs && \
    echo "pub struct ServerConfig {" >> src/config.rs && \
    echo "    pub grpc_port: u16," >> src/config.rs && \
    echo "    pub dictionary: Vec<String>," >> src/config.rs && \
    echo "}" >> src/config.rs && \
    echo "impl ServerConfig {" >> src/config.rs && \
    echo "    pub fn from_env() -> Result<Self> { Ok(Self { grpc_port: 50051, dictionary: vec![] }) }" >> src/config.rs && \
    echo "}" >> src/config.rs && \
    echo "pub struct Dictionary;" > src/dictionary.rs && \
    echo "impl Dictionary {" >> src/dictionary.rs && \
    echo "    pub fn new(_v: Vec<String>) -> Self { Self }" >> src/dictionary.rs && \
    echo "}" >> src/dictionary.rs && \
    echo "use crate::config::ServerConfig;" > src/model.rs && \
    echo "pub struct Model;" >> src/model.rs && \
    echo "impl Model {" >> src/model.rs && \
    echo "    pub fn new(_c: ServerConfig) -> Self { Self }" >> src/model.rs && \
    echo "    pub fn is_available(&self) -> bool { true }" >> src/model.rs && \
    echo "}" >> src/model.rs && \
    echo "use crate::model::Model;" > src/transcription.rs && \
    echo "use crate::dictionary::Dictionary;" >> src/transcription.rs && \
    echo "use crate::config::ServerConfig;" >> src/transcription.rs && \
    echo "use anyhow::Result;" >> src/transcription.rs && \
    echo "use std::sync::Arc;" >> src/transcription.rs && \
    echo "pub struct TranscriptionService;" >> src/transcription.rs && \
    echo "impl TranscriptionService {" >> src/transcription.rs && \
    echo "    pub fn new(_m: Arc<Model>, _d: Option<Arc<Dictionary>>, _c: Arc<ServerConfig>) -> Result<Self> { Ok(Self) }" >> src/transcription.rs && \
    echo "    pub fn transcribe_audio_bytes(&self, _data: &[u8]) -> Result<String> { Ok(String::new()) }" >> src/transcription.rs && \
    echo "}" >> src/transcription.rs && \
    echo "pub fn placeholder() {}" > src/engine/mod.rs && \
    cargo build --release --lib

# Build server dependencies (keep library dummy sources for this step)
WORKDIR /app/murmure-server
RUN mkdir -p src/server && \
    echo "mod server;" > src/main.rs && \
    echo "use server::TranscriptionServiceImpl;" >> src/main.rs && \
    echo "use server::murmure;" >> src/main.rs && \
    echo "#[tokio::main]" >> src/main.rs && \
    echo "async fn main() -> anyhow::Result<()> { Ok(()) }" >> src/main.rs && \
    echo "pub mod grpc;" > src/server/mod.rs && \
    echo "pub use grpc::{TranscriptionServiceImpl, murmure};" >> src/server/mod.rs && \
    echo "pub mod murmure {}" > src/server/grpc.rs && \
    echo "pub struct TranscriptionServiceImpl;" >> src/server/grpc.rs && \
    echo "impl TranscriptionServiceImpl {" >> src/server/grpc.rs && \
    echo "    pub fn new(_service: std::sync::Arc<murmure_lib::TranscriptionService>) -> Self { Self }" >> src/server/grpc.rs && \
    echo "}" >> src/server/grpc.rs && \
    cargo build --release

# Now copy actual source code and rebuild
COPY murmure-lib/src ./../murmure-lib/src
COPY murmure-server/src ./src

# Build the final application with real sources
WORKDIR /app/murmure-server
# Clean previous build to ensure we're using real sources, not cached dummy ones
RUN cargo clean --release || true
RUN cargo build --release

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
COPY --from=builder /app/murmure-server/target/release/murmure-server /usr/local/bin/murmure-server

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
