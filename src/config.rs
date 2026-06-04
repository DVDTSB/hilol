use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::{Result, anyhow};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub version: String,
    pub base_url: String,
    #[serde(default = "default_output")]
    pub output_dir: String,
    #[serde(default = "default_content")]
    pub content_dir: String,
    #[serde(default = "default_templates")]
    pub template_dir: String,
}

fn default_output() -> String {
    "public".to_string()
}

fn default_content() -> String {
    "pages".to_string()
}

fn default_templates() -> String {
    "templates".to_string()
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        toml::from_str(&content).map_err(|e| anyhow!(":( invalid config.toml: {}", e))
    }
}
