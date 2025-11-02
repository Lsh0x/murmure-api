use crate::transcription::TranscriptionService;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

// Include the generated proto code
pub mod murmure {
    include!(concat!(env!("OUT_DIR"), "/murmure.rs"));
}

use murmure::{
    transcription_service_server, TranscribeFileRequest, TranscribeFileResponse,
    TranscribeStreamRequest, TranscribeStreamResponse,
};

pub struct TranscriptionServiceImpl {
    service: Arc<TranscriptionService>,
}

impl TranscriptionServiceImpl {
    pub fn new(service: Arc<TranscriptionService>) -> Self {
        Self { service }
    }
}

#[tonic::async_trait]
impl murmure::transcription_service_server::TranscriptionService for TranscriptionServiceImpl {
    async fn transcribe_file(
        &self,
        request: Request<TranscribeFileRequest>,
    ) -> Result<Response<TranscribeFileResponse>, Status> {
        let req = request.into_inner();
        let audio_data = req.audio_data;

        tracing::debug!("Received transcribe_file request: {} bytes", audio_data.len());

        match self.service.transcribe_audio_bytes(&audio_data) {
            Ok(text) => {
                tracing::info!("Transcription successful: {} chars", text.len());
                Ok(Response::new(TranscribeFileResponse {
                    text,
                    success: true,
                    error: String::new(),
                }))
            }
            Err(e) => {
                tracing::error!("Transcription failed: {}", e);
                Ok(Response::new(TranscribeFileResponse {
                    text: String::new(),
                    success: false,
                    error: format!("Transcription failed: {}", e),
                }))
            }
        }
    }

    type TranscribeStreamStream = ReceiverStream<Result<TranscribeStreamResponse, Status>>;

    async fn transcribe_stream(
        &self,
        request: Request<tonic::Streaming<TranscribeStreamRequest>>,
    ) -> Result<Response<Self::TranscribeStreamStream>, Status> {
        let mut stream = request.into_inner();
        let (tx, rx) = mpsc::channel(128);

        let service = Arc::clone(&self.service);
        
        tokio::spawn(async move {
            let mut audio_buffer = Vec::new();
            let mut end_of_stream = false;

            while let Some(result) = stream.message().await.transpose() {
                match result {
                    Ok(req) => {
                        match req.request_type {
                            Some(murmure::transcribe_stream_request::RequestType::AudioChunk(chunk)) => {
                                audio_buffer.extend_from_slice(&chunk);
                            }
                            Some(murmure::transcribe_stream_request::RequestType::EndOfStream(_)) => {
                                end_of_stream = true;
                                break;
                            }
                            None => {
                                // Empty request, ignore
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx
                            .send(Ok(TranscribeStreamResponse {
                                response_type: Some(
                                    murmure::transcribe_stream_response::ResponseType::Error(
                                        format!("Stream error: {}", e),
                                    ),
                                ),
                                is_final: false,
                            }))
                            .await;
                        return;
                    }
                }
            }

            // Process accumulated audio buffer
            if !audio_buffer.is_empty() || end_of_stream {
                match service.transcribe_audio_bytes(&audio_buffer) {
                    Ok(text) => {
                        let response = TranscribeStreamResponse {
                            response_type: Some(murmure::transcribe_stream_response::ResponseType::FinalText(
                                text,
                            )),
                            is_final: true,
                        };
                        let _ = tx.send(Ok(response)).await;
                    }
                    Err(e) => {
                        let response = TranscribeStreamResponse {
                            response_type: Some(murmure::transcribe_stream_response::ResponseType::Error(
                                format!("Transcription failed: {}", e),
                            )),
                            is_final: true,
                        };
                        let _ = tx.send(Ok(response)).await;
                    }
                }
            }

            // Signal end of response stream
            drop(tx);
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

