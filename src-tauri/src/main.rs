use murmure_lib::*;
use murmure_lib::server::grpc::murmure;
use std::sync::Arc;
use tokio::signal;
use tonic::transport::Server;
use tracing::{info, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting Murmure gRPC Server...");

    // Load configuration
    let config = Arc::new(ServerConfig::from_env()?);
    info!("Configuration loaded: gRPC port = {}", config.grpc_port);

    // Initialize model
    let model = Arc::new(Model::new((*config).clone()));
    if !model.is_available() {
        error!("Model is not available. Please ensure MURMURE_MODEL_PATH is set correctly.");
        anyhow::bail!("Model not available");
    }
    info!("Model initialized");

    // Initialize dictionary (optional)
    let dictionary = if !config.dictionary.is_empty() {
        Some(Arc::new(Dictionary::new(config.dictionary.clone())))
    } else {
        None
    };
    if dictionary.is_some() {
        info!("Custom dictionary loaded with {} words", config.dictionary.len());
    }

    // Create transcription service
    let transcription_service = Arc::new(
        TranscriptionService::new(model, dictionary, config.clone())
            .map_err(|e| anyhow::anyhow!("Failed to initialize transcription service: {}", e))?,
    );
    info!("Transcription service ready");

    // Create gRPC service
    let grpc_service = TranscriptionServiceImpl::new(transcription_service);

    // Create gRPC server
    let addr = format!("0.0.0.0:{}", config.grpc_port).parse()?;
    info!("gRPC server listening on {}", addr);

    Server::builder()
        .add_service(
            murmure::transcription_service_server::TranscriptionServiceServer::new(
                grpc_service,
            ),
        )
        .serve_with_shutdown(addr, async {
            // Wait for shutdown signal
            match signal::ctrl_c().await {
                Ok(()) => {
                    info!("Shutdown signal received");
                }
                Err(err) => {
                    error!("Unable to listen for shutdown signal: {}", err);
                }
            }
        })
        .await?;

    info!("Server shut down");
    Ok(())
}
