use super::audio::write_wav_bytes;
use super::engine::piper::{PiperEngine, PiperModelParams};
use super::engine::synthesis_engine::SynthesisEngine;
use super::model::TtsModel;
use anyhow::Result;
use parking_lot::Mutex;
use std::sync::Arc;

pub struct SynthesisStream {
    engine: Arc<Mutex<PiperEngine>>,
    text_buffer: String,
    sample_rate: u32,
}

impl SynthesisStream {
    pub fn new(model: Arc<TtsModel>) -> Result<Self> {
        let model_path = model
            .get_model_path()
            .map_err(|e| anyhow::anyhow!("Failed to get model path: {}", e))?;

        let mut engine = PiperEngine::new();
        engine
            .load_model_with_params(&model_path, PiperModelParams::default())
            .map_err(|e| anyhow::anyhow!("Failed to load TTS model: {}", e))?;

        Ok(Self {
            engine: Arc::new(Mutex::new(engine)),
            text_buffer: String::new(),
            sample_rate: 22050,
        })
    }

    pub fn push_text(&mut self, text: &str) -> Result<()> {
        self.text_buffer.push_str(text);
        Ok(())
    }

    pub fn flush(&mut self) -> Result<Vec<u8>> {
        if self.text_buffer.is_empty() {
            return Ok(Vec::new());
        }

        let text_to_synthesize = self.text_buffer.clone();
        self.text_buffer.clear();

        let mut engine = self.engine.lock();
        let result = engine
            .synthesize_incremental(&text_to_synthesize, false, None)
            .map_err(|e| anyhow::anyhow!("Synthesis failed: {}", e))?;

        self.sample_rate = result.sample_rate;
        write_wav_bytes(&result.audio_samples, result.sample_rate)
    }

    pub fn synthesize_chunk(&mut self, text: &str) -> Result<Vec<u8>> {
        let mut engine = self.engine.lock();
        let result = engine
            .synthesize_incremental(text, false, None)
            .map_err(|e| anyhow::anyhow!("Synthesis failed: {}", e))?;

        self.sample_rate = result.sample_rate;
        write_wav_bytes(&result.audio_samples, result.sample_rate)
    }

    pub fn finalize(&mut self) -> Result<Vec<u8>> {
        if self.text_buffer.is_empty() {
            return Ok(Vec::new());
        }

        let text_to_synthesize = self.text_buffer.clone();
        self.text_buffer.clear();

        let mut engine = self.engine.lock();
        let result = engine
            .synthesize_incremental(&text_to_synthesize, true, None)
            .map_err(|e| anyhow::anyhow!("Synthesis failed: {}", e))?;

        self.sample_rate = result.sample_rate;
        write_wav_bytes(&result.audio_samples, result.sample_rate)
    }
}
