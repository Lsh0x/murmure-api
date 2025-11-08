use crate::stt::audio::{preload_engine, transcribe_audio};
use crate::stt::config::ServerConfig;
use crate::stt::dictionary::Dictionary;
use crate::stt::model::Model;
use anyhow::Result;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use tempfile::NamedTempFile;

pub struct TranscriptionService {
    model: Arc<Model>,
    dictionary: Option<Arc<Dictionary>>,
    config: Arc<ServerConfig>,
    engine_loaded: Arc<std::sync::atomic::AtomicBool>,
}

impl TranscriptionService {
    pub fn new(
        model: Arc<Model>,
        dictionary: Option<Arc<Dictionary>>,
        config: Arc<ServerConfig>,
    ) -> Result<Self> {
        let service = Self {
            model,
            dictionary,
            config,
            engine_loaded: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        };

        // Preload engine on initialization
        service.ensure_engine_loaded()?;

        Ok(service)
    }

    fn ensure_engine_loaded(&self) -> Result<()> {
        if !self
            .engine_loaded
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            preload_engine(&self.model)?;
            self.engine_loaded
                .store(true, std::sync::atomic::Ordering::Relaxed);
        }
        Ok(())
    }

    pub fn transcribe_audio_bytes(&self, audio_data: &[u8]) -> Result<String> {
        // Ensure engine is loaded
        self.ensure_engine_loaded()?;

        // Write audio data to temporary file
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(audio_data)?;
        temp_file.flush()?;
        let temp_path = temp_file.path();

        // Transcribe
        let result = transcribe_audio(
            temp_path,
            &self.model,
            self.dictionary.as_deref(),
            &self.config,
        )?;

        Ok(result)
    }

    pub fn transcribe_audio_file(&self, audio_path: &Path) -> Result<String> {
        // Ensure engine is loaded
        self.ensure_engine_loaded()?;

        // Transcribe
        let result = transcribe_audio(
            audio_path,
            &self.model,
            self.dictionary.as_deref(),
            &self.config,
        )?;

        Ok(result)
    }

    pub fn get_model(&self) -> &Arc<Model> {
        &self.model
    }

    pub fn get_dictionary(&self) -> Option<&Arc<Dictionary>> {
        self.dictionary.as_ref()
    }

    pub fn get_config(&self) -> &Arc<ServerConfig> {
        &self.config
    }
}
