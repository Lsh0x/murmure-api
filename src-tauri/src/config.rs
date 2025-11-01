use serde::{Deserialize, Serialize};
use std::{env, fs, path::PathBuf};
use anyhow::{Context, Result};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct ServerConfig {
    pub model_path: Option<PathBuf>,
    pub cc_rules_path: Option<PathBuf>,
    pub dictionary: Vec<String>,
    pub grpc_port: u16,
    pub log_level: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            model_path: None,
            cc_rules_path: None,
            dictionary: Vec::new(),
            grpc_port: 50051,
            log_level: "info".to_string(),
        }
    }
}

impl ServerConfig {
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();

        // Load from environment variables
        if let Ok(model_path) = env::var("MURMURE_MODEL_PATH") {
            config.model_path = Some(PathBuf::from(model_path));
        }

        if let Ok(cc_rules_path) = env::var("MURMURE_CC_RULES_PATH") {
            config.cc_rules_path = Some(PathBuf::from(cc_rules_path));
        }

        if let Ok(dict_json) = env::var("MURMURE_DICTIONARY") {
            config.dictionary = serde_json::from_str(&dict_json)
                .context("Failed to parse MURMURE_DICTIONARY as JSON array")?;
        }

        if let Ok(port_str) = env::var("MURMURE_GRPC_PORT") {
            config.grpc_port = port_str
                .parse()
                .context("MURMURE_GRPC_PORT must be a valid port number")?;
        }

        if let Ok(log_level) = env::var("MURMURE_LOG_LEVEL") {
            config.log_level = log_level;
        }

        // Try to load from config file (optional)
        if let Some(file_config) = Self::load_from_file("config.json")
            .or_else(|| Self::load_from_file("config.toml")) {
            // Merge file config with env config (env takes precedence)
            config = file_config.merge_with_env(config);
        }

        Ok(config)
    }

    fn load_from_file(path: &str) -> Option<Self> {
        if let Ok(content) = fs::read_to_string(path) {
            if path.ends_with(".json") {
                serde_json::from_str(&content).ok()
            } else if path.ends_with(".toml") {
                // Try to parse as TOML, but don't fail if it doesn't work
                match toml::from_str(&content) {
                    Ok(config) => Some(config),
                    Err(e) => {
                        eprintln!("Warning: Failed to parse TOML config file {}: {}", path, e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn merge_with_env(self, env_config: Self) -> Self {
        Self {
            model_path: env_config.model_path.or(self.model_path),
            cc_rules_path: env_config.cc_rules_path.or(self.cc_rules_path),
            dictionary: if env_config.dictionary.is_empty() {
                self.dictionary
            } else {
                env_config.dictionary
            },
            grpc_port: env_config.grpc_port,
            log_level: env_config.log_level,
        }
    }

    pub fn get_model_path(&self) -> Result<PathBuf> {
        if let Some(ref path) = self.model_path {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        // Try default locations
        const MODEL_FILENAME: &str = "parakeet-tdt-0.6b-v3-int8";
        
        let possible_paths = vec![
            PathBuf::from(format!("resources/{}", MODEL_FILENAME)),
            PathBuf::from(format!("../resources/{}", MODEL_FILENAME)),
            PathBuf::from(format!("_up_/resources/{}", MODEL_FILENAME)),
        ];

        for path in possible_paths {
            if path.exists() {
                println!("Model found at: {}", path.display());
                return Ok(path);
            }
        }

        // Try relative to executable
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let dev_path = exe_dir.join("_up_").join("resources").join(MODEL_FILENAME);
                if dev_path.exists() {
                    println!("Model found at dev location: {}", dev_path.display());
                    return Ok(dev_path);
                }

                let resource_path = exe_dir.join("resources").join(MODEL_FILENAME);
                if resource_path.exists() {
                    println!("Model found at: {}", resource_path.display());
                    return Ok(resource_path);
                }
            }
        }

        anyhow::bail!(
            "Model '{}' not found. Set MURMURE_MODEL_PATH environment variable or place model in resources/ directory.",
            MODEL_FILENAME
        )
    }

    pub fn get_cc_rules_path(&self) -> Result<PathBuf> {
        if let Some(ref path) = self.cc_rules_path {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        // Try default locations
        let possible_paths = vec![
            PathBuf::from("resources/cc-rules"),
            PathBuf::from("../resources/cc-rules"),
            PathBuf::from("_up_/resources/cc-rules"),
        ];

        for path in possible_paths {
            if path.exists() {
                println!("CC rules found at: {}", path.display());
                return Ok(path);
            }
        }

        // Try relative to executable
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let resource_path = exe_dir.join("resources").join("cc-rules");
                if resource_path.exists() {
                    println!("CC rules found at: {}", resource_path.display());
                    return Ok(resource_path);
                }
            }
        }

        anyhow::bail!(
            "CC rules directory not found. Set MURMURE_CC_RULES_PATH environment variable or place cc-rules in resources/ directory."
        )
    }
}

