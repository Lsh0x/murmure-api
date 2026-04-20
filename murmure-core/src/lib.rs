// Murmure Core Library
// Unified library for Speech-To-Text (STT) and Text-To-Speech (TTS)

pub mod stt;
pub mod tts;

// Re-export STT types for backward compatibility
pub use stt::{Dictionary, Model, ServerConfig, TranscriptionService};

// Re-export TTS types
pub use tts::{SynthesisService, SynthesisStream, TtsConfig, TtsModel};
