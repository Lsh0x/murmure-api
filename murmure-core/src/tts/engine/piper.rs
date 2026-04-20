use super::synthesis_engine::{SynthesisEngine, SynthesisResult};
use std::path::Path;
use piper_tts_rust::model_handler::Model;

pub struct PiperModelParams {
    pub use_cuda: bool,
}

impl Default for PiperModelParams {
    fn default() -> Self {
        Self { use_cuda: false }
    }
}

pub struct PiperInferenceParams {
    pub speaker_id: Option<u32>,
    pub speed: f32,
}

impl Default for PiperInferenceParams {
    fn default() -> Self {
        Self {
            speaker_id: None,
            speed: 1.0,
        }
    }
}

pub struct PiperEngine {
    model: Option<Model>,
    sample_rate: u32,
}

impl Default for PiperEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PiperEngine {
    pub fn new() -> Self {
        Self {
            model: None,
            sample_rate: 22050,
        }
    }
}

impl SynthesisEngine for PiperEngine {
    type ModelParams = PiperModelParams;
    type InferenceParams = PiperInferenceParams;

    fn load_model_with_params(
        &mut self,
        model_path: &Path,
        _params: Self::ModelParams,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Find the .onnx file and .json config file in the directory
        let (onnx_file, config_file) = if model_path.is_file() && model_path.extension().and_then(|s| s.to_str()) == Some("onnx") {
            let config_path = model_path.with_extension("onnx.json");
            if !config_path.exists() {
                return Err(format!("Config file not found: {}", config_path.display()).into());
            }
            (model_path.to_path_buf(), config_path)
        } else if model_path.is_dir() {
            // Look for .onnx and .json files in the directory
            let mut onnx_path = None;
            let mut json_path = None;
            for entry in std::fs::read_dir(model_path)? {
                let entry = entry?;
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if ext == "onnx" {
                        onnx_path = Some(path.clone());
                        // Look for corresponding .json file
                        let json_file = path.with_extension("onnx.json");
                        if json_file.exists() {
                            json_path = Some(json_file);
                        }
                    }
                }
            }
            let onnx = onnx_path.ok_or_else(|| "No .onnx file found in model directory".to_string())?;
            let json = json_path.ok_or_else(|| "No .onnx.json config file found in model directory".to_string())?;
            (onnx, json)
        } else {
            return Err("Model path must be a directory or .onnx file".into());
        };

        // Load the model (requires both .onnx and .json paths)
        let model = Model::new(
            onnx_file.to_str().ok_or_else(|| "Invalid model path")?,
            config_file.to_str().ok_or_else(|| "Invalid config path")?,
        )
        .map_err(|e| format!("Failed to load Piper model: {}", e))?;
        
        // Get sample rate from the model config (convert u64 to u32)
        self.sample_rate = model.config.audio.sample_rate as u32;
        self.model = Some(model);

        Ok(())
    }

    fn synthesize_text(
        &mut self,
        text: &str,
        _params: Option<Self::InferenceParams>,
    ) -> Result<SynthesisResult, Box<dyn std::error::Error>> {
        let model = self.model.as_mut()
            .ok_or_else(|| "Model not loaded".to_string())?;

        // Convert text to IPA phonemes first, then synthesize
        // For now, try using process_ipa_string with the text directly
        // If that doesn't work, we'll need to add PhonemeGen for text-to-IPA conversion
        // Note: process_ipa_string expects IPA format, but let's try with regular text first
        // The model might handle text-to-IPA conversion internally
        
        // Try to synthesize - if it fails, we may need PhonemeGen
        let (_shape, audio_samples) = model.process_ipa_string(text)
            .map_err(|e| format!("Synthesis failed (text may need IPA conversion): {}", e))?;

        Ok(SynthesisResult {
            audio_samples,
            sample_rate: self.sample_rate,
            is_final: true,
        })
    }

    fn synthesize_incremental(
        &mut self,
        text: &str,
        is_final: bool,
        params: Option<Self::InferenceParams>,
    ) -> Result<SynthesisResult, Box<dyn std::error::Error>> {
        // For incremental synthesis, synthesize the text chunk
        self.synthesize_text(text, params).map(|mut result| {
            result.is_final = is_final;
            result
        })
    }

    fn unload_model(&mut self) {
        self.model = None;
    }
}
