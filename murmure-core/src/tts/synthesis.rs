use super::audio::write_wav_bytes;
use super::config::TtsConfig;
use super::engine::piper::{PiperEngine, PiperModelParams};
use super::engine::synthesis_engine::SynthesisEngine;
use super::model::TtsModel;
use anyhow::Result;
use parking_lot::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

static ENGINE: once_cell::sync::Lazy<Mutex<Option<PiperEngine>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(None));

pub struct SynthesisService {
    model: Arc<TtsModel>,
    config: Arc<TtsConfig>,
    engine_loaded: Arc<AtomicBool>,
}

impl SynthesisService {
    pub fn new(model: Arc<TtsModel>, config: Arc<TtsConfig>) -> Result<Self> {
        let service = Self {
            model,
            config,
            engine_loaded: Arc::new(AtomicBool::new(false)),
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
            let model_path = self
                .model
                .get_model_path()
                .map_err(|e| anyhow::anyhow!("Failed to get model path: {}", e))?;

            let mut engine_guard = ENGINE.lock();
            if engine_guard.is_none() {
                let mut new_engine = PiperEngine::new();
                new_engine
                    .load_model_with_params(&model_path, PiperModelParams::default())
                    .map_err(|e| anyhow::anyhow!("Failed to load TTS model: {}", e))?;
                *engine_guard = Some(new_engine);
            }
            self.engine_loaded
                .store(true, std::sync::atomic::Ordering::Relaxed);
        }
        Ok(())
    }

    pub fn synthesize_text(&self, text: &str) -> Result<Vec<u8>> {
        self.ensure_engine_loaded()?;

        let mut engine_guard = ENGINE.lock();
        let engine = engine_guard
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Engine not loaded"))?;

        let result = engine
            .synthesize_text(text, None)
            .map_err(|e| anyhow::anyhow!("Synthesis failed: {}", e))?;

        // Convert audio samples to WAV bytes
        let wav_bytes = write_wav_bytes(&result.audio_samples, result.sample_rate)?;

        Ok(wav_bytes)
    }

    pub fn get_model(&self) -> &Arc<TtsModel> {
        &self.model
    }

    pub fn get_config(&self) -> &Arc<TtsConfig> {
        &self.config
    }
}
