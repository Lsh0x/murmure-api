// TTS module placeholder - to be implemented
pub mod audio;
pub mod config;
pub mod engine;
pub mod model;
pub mod stream;
pub mod synthesis;

// Re-export public types
pub use config::TtsConfig;
pub use model::TtsModel;
pub use stream::SynthesisStream;
pub use synthesis::SynthesisService;
