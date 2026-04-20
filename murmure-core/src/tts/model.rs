use super::config::TtsConfig;
use anyhow::Result;
use std::path::PathBuf;

pub struct TtsModel {
    config: TtsConfig,
}

impl TtsModel {
    pub fn new(config: TtsConfig) -> Self {
        Self { config }
    }

    pub fn get_model_path(&self) -> Result<PathBuf> {
        self.config.get_model_path()
    }

    pub fn is_available(&self) -> bool {
        self.get_model_path().is_ok()
    }

    pub fn get_config(&self) -> &TtsConfig {
        &self.config
    }
}
