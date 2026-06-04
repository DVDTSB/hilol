use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::Result;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CacheEntry {
    pub content_hash: String,
    pub template_hash: String,
    pub built_at: u64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Cache {
    entries: HashMap<String, CacheEntry>,
}

impl Cache {
    pub fn load(cache_path: &Path) -> Result<Self> {
        if !cache_path.exists() {
            return Ok(Cache::default());
        }
        let content = fs::read_to_string(cache_path)?;
        Ok(serde_json::from_str(&content).unwrap_or_default())
    }

    pub fn save(&self, cache_path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(&self)?;
        fs::write(cache_path, content)?;
        Ok(())
    }

    pub fn is_dirty(&self, page_path: &str, content_hash: &str, template_hash: &str) -> bool {
        match self.entries.get(page_path) {
            None => true,
            Some(e) => e.content_hash != content_hash || e.template_hash != template_hash,
        }
    }

    pub fn update(&mut self, page_path: String, content_hash: String, template_hash: String) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.entries.insert(
            page_path,
            CacheEntry {
                content_hash,
                template_hash,
                built_at: now,
            },
        );
    }
}

pub fn hash_file(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)?;
    Ok(hash_string(&content))
}

pub fn hash_string(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}
