use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{env, fs, path::PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct TtsConfig {
    pub model_path: Option<PathBuf>,
    pub sample_rate: u32,
    pub speaker_id: Option<u32>,
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            model_path: None,
            sample_rate: 22050,
            speaker_id: None,
        }
    }
}

impl TtsConfig {
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();

        // Load from environment variables
        if let Ok(model_path) = env::var("MURMURE_TTS_MODEL_PATH") {
            config.model_path = Some(PathBuf::from(model_path));
        }

        if let Ok(sample_rate_str) = env::var("MURMURE_TTS_SAMPLE_RATE") {
            config.sample_rate = sample_rate_str
                .parse()
                .context("MURMURE_TTS_SAMPLE_RATE must be a valid number")?;
        }

        if let Ok(speaker_id_str) = env::var("MURMURE_TTS_SPEAKER_ID") {
            config.speaker_id = Some(
                speaker_id_str
                    .parse()
                    .context("MURMURE_TTS_SPEAKER_ID must be a valid number")?,
            );
        }

        // Try to load from config file (optional)
        if let Some(file_config) =
            Self::load_from_file("config.json").or_else(|| Self::load_from_file("config.toml"))
        {
            // Merge file config with env config (env takes precedence)
            if config.model_path.is_none() {
                config.model_path = file_config.model_path;
            }
            if config.sample_rate == Self::default().sample_rate {
                config.sample_rate = file_config.sample_rate;
            }
            if config.speaker_id.is_none() {
                config.speaker_id = file_config.speaker_id;
            }
        }

        Ok(config)
    }

    fn load_from_file(path: &str) -> Option<Self> {
        if let Ok(content) = fs::read_to_string(path) {
            if path.ends_with(".json") {
                if let Ok(mut parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                    // Extract TTS config from nested structure if it exists
                    if let Some(tts_config) = parsed.get_mut("tts") {
                        serde_json::from_value(tts_config.take()).ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else if path.ends_with(".toml") {
                // TOML parsing would go here if needed
                None
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_model_path(&self) -> Result<PathBuf> {
        if let Some(ref path) = self.model_path {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        // Try default locations
        let default_paths = vec![
            PathBuf::from("resources/tts"),
            PathBuf::from("../resources/tts"),
            PathBuf::from("_up_/resources/tts"),
            // Legacy path for backward compatibility
            PathBuf::from("resources/piper-model"),
        ];

        for path in default_paths {
            if path.exists() {
                return Ok(path);
            }
        }

        Err(anyhow::anyhow!(
            "TTS model path not found. Set MURMURE_TTS_MODEL_PATH environment variable or place model in resources/tts/"
        ))
    }
}
