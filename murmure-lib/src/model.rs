use crate::config::ServerConfig;
use anyhow::Result;
use std::path::PathBuf;

const MODEL_FILENAME: &str = "parakeet-tdt-0.6b-v3-int8";

pub struct Model {
    config: ServerConfig,
}

impl Model {
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }

    pub fn get_model_path(&self) -> Result<PathBuf> {
        self.config.get_model_path()
    }

    pub fn is_available(&self) -> bool {
        self.get_model_path().is_ok()
    }
}
