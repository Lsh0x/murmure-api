// Placeholder - to be implemented
use std::path::Path;

pub struct SynthesisResult {
    pub audio_samples: Vec<f32>,
    pub sample_rate: u32,
    pub is_final: bool,
}

pub trait SynthesisEngine {
    type ModelParams: Default;
    type InferenceParams: Default;

    fn load_model_with_params(
        &mut self,
        _model_path: &Path,
        _params: Self::ModelParams,
    ) -> Result<(), Box<dyn std::error::Error>>;

    fn synthesize_text(
        &mut self,
        _text: &str,
        _params: Option<Self::InferenceParams>,
    ) -> Result<SynthesisResult, Box<dyn std::error::Error>>;

    fn synthesize_incremental(
        &mut self,
        _text: &str,
        _is_final: bool,
        _params: Option<Self::InferenceParams>,
    ) -> Result<SynthesisResult, Box<dyn std::error::Error>>;

    fn unload_model(&mut self);
}
