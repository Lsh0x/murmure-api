use std::sync::Arc;
use tonic::transport::Server;
use tracing::{error, info};

use murmure_core::stt::config::ServerConfig;
use murmure_core::stt::dictionary::Dictionary;
use murmure_core::stt::model::Model;
use murmure_core::stt::transcription::TranscriptionService;
use murmure_core::tts::config::TtsConfig;
use murmure_core::tts::model::TtsModel;
use murmure_core::tts::synthesis::SynthesisService;

mod server;

use server::murmure;
use server::{SynthesisServiceImpl, TranscriptionServiceImpl};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Print to stderr immediately to ensure we see output even if logging fails
    eprintln!("[DEBUG] Starting Murmure server...");

    // Initialize logging - ensure output goes to stdout
    tracing_subscriber::fmt()
        .with_writer(std::io::stdout)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting Murmure gRPC Server...");

    // Load configuration
    let config = match ServerConfig::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[ERROR] Failed to load configuration: {}", e);
            return Err(e);
        }
    };
    let config = Arc::new(config);
    info!("Configuration loaded: gRPC port = {}", config.grpc_port);

    // Initialize model
    eprintln!("[DEBUG] Checking model availability...");
    let model = Arc::new(Model::new((*config).clone()));
    if !model.is_available() {
        eprintln!("[ERROR] Model is not available. Ensure MURMURE_MODEL_PATH is set correctly.");
        error!("Model is not available. Please ensure MURMURE_MODEL_PATH is set correctly.");
        anyhow::bail!("Model not available");
    }
    eprintln!("[DEBUG] Model is available");
    info!("Model initialized");

    // Initialize dictionary (optional)
    let dictionary = if !config.dictionary.is_empty() {
        Some(Arc::new(Dictionary::new(config.dictionary.clone())))
    } else {
        None
    };
    if dictionary.is_some() {
        info!(
            "Custom dictionary loaded with {} words",
            config.dictionary.len()
        );
    }

    // Create transcription service
    let transcription_service = Arc::new(
        TranscriptionService::new(model, dictionary, config.clone())
            .map_err(|e| anyhow::anyhow!("Failed to initialize transcription service: {}", e))?,
    );
    info!("Transcription service ready");

    // Create gRPC transcription service
    let grpc_transcription_service = TranscriptionServiceImpl::new(transcription_service);

    // Initialize TTS service (optional)
    let grpc_synthesis_service = match TtsConfig::from_env() {
        Ok(tts_config) => {
            let tts_model = Arc::new(TtsModel::new(tts_config.clone()));
            match SynthesisService::new(tts_model, Arc::new(tts_config)) {
                Ok(tts_service) => {
                    info!("TTS service ready");
                    Some(SynthesisServiceImpl::new(Arc::new(tts_service)))
                }
                Err(e) => {
                    info!("TTS service not available: {} (continuing without TTS)", e);
                    None
                }
            }
        }
        Err(e) => {
            info!("TTS configuration not found: {} (continuing without TTS)", e);
            None
        }
    };

    // Create gRPC server
    eprintln!("[DEBUG] Creating gRPC server...");
    let addr = format!("0.0.0.0:{}", config.grpc_port).parse()?;
    eprintln!("[DEBUG] gRPC server will listen on {}", addr);
    info!("gRPC server listening on {}", addr);

    eprintln!("[DEBUG] About to start server, binding to {}", addr);
    info!("Starting server on {}", addr);

    // Create shutdown signal receiver
    // Note: In Docker (PID 1), signals must be handled explicitly
    #[cfg(unix)]
    let shutdown = async {
        use tokio::signal::unix::{signal, SignalKind};

        // Create signal handlers for both SIGTERM (Docker stop) and SIGINT
        // If signal creation fails, we panic because we can't run without signal handling
        let mut sigterm = signal(SignalKind::terminate())
            .expect("Failed to create SIGTERM handler - cannot run server without signal handling");

        let mut sigint = signal(SignalKind::interrupt())
            .expect("Failed to create SIGINT handler - cannot run server without signal handling");

        eprintln!("[DEBUG] Signal handlers installed, waiting for shutdown signal...");
        info!("Server is ready and listening for requests");

        // Wait for either signal - this will block until one is received
        tokio::select! {
            result = sigint.recv() => {
                match result {
                    Some(_) => {
                        eprintln!("[DEBUG] SIGINT received");
                        info!("SIGINT received, shutting down gracefully");
                    }
                    None => {
                        eprintln!("[WARN] SIGINT stream ended unexpectedly");
                    }
                }
            }
            result = sigterm.recv() => {
                match result {
                    Some(_) => {
                        eprintln!("[DEBUG] SIGTERM received");
                        info!("SIGTERM received, shutting down gracefully");
                    }
                    None => {
                        eprintln!("[WARN] SIGTERM stream ended unexpectedly");
                    }
                }
            }
        }

        eprintln!("[DEBUG] Shutdown signal processed, server will stop");
    };

    #[cfg(not(unix))]
    let shutdown = async {
        eprintln!("[DEBUG] Setting up Ctrl+C handler...");
        info!("Server is ready and listening for requests");
        signal::ctrl_c()
            .await
            .expect("Failed to listen for shutdown signal");
        eprintln!("[DEBUG] Ctrl+C received");
        info!("Shutdown signal received");
    };

    eprintln!("[DEBUG] Building server...");
    
    // Build server with transcription service (always available)
    let mut server = Server::builder()
        .add_service(
            murmure::transcription_service_server::TranscriptionServiceServer::new(
                grpc_transcription_service,
            ),
        );
    
    // Add synthesis service (if available)
    if let Some(synthesis_service) = grpc_synthesis_service {
        server = server.add_service(
            murmure::synthesis_service_server::SynthesisServiceServer::new(synthesis_service),
        );
        info!("TTS gRPC service registered");
    }

    eprintln!("[DEBUG] Starting server with shutdown handler...");

    // Start the server - this will block until shutdown signal is received
    // Use tokio::select to handle shutdown signal
    tokio::select! {
        result = server.serve(addr) => {
            match result {
                Ok(_) => {
                    eprintln!("[DEBUG] Server exited normally");
                    info!("Server shut down");
                }
                Err(e) => {
                    eprintln!("[ERROR] Server error: {}", e);
                    error!("Server error: {}", e);
                    return Err(anyhow::anyhow!("Server failed: {}", e));
                }
            }
        }
        _ = shutdown => {
            eprintln!("[DEBUG] Shutdown signal received, stopping server");
            info!("Shutdown signal received");
        }
    }

    info!("Server shut down");
    Ok(())
}
