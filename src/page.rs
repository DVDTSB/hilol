
use serde::{Serialize,Deserialize};
use anyhow::{Result, anyhow};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Meta {
    pub title: String,
    pub date: Option<String>,
    pub template: Option<String>,
    pub description: Option<String>, 
    pub tags: Option<Vec<String>>,
    pub draft: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Page{
    pub meta: Meta,
    pub content: String,
    pub slug: String,
    pub path: String,
}

fn normalize_slug(file: &str) -> String {
    file.to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

pub fn parse(file: &str, source: &str) -> Result<Page> {
    let (toml_str, body) = split_meta(source);
    let meta = toml::from_str::<Meta>(&toml_str)
        .map_err(|e| anyhow!("failed to parse frontmatter in {}: {}", file, e))?;
    
    let slug = normalize_slug(file);
    
    Ok(Page {
        meta,
        content: body,
        slug,
        path: file.to_string(),
    })
}

fn split_meta(source: &str) -> (String, String) {
    let mut lines = source.lines();
    let mut toml_str = String::new();
    let mut in_toml = false;

    for line in lines.by_ref() {
        if line.trim() == "---" || line.trim() == "+++" {
            if in_toml {
                break;
            } else {
                in_toml = true; 
                continue;
            }
        }
        if in_toml {
            toml_str.push_str(line);
            toml_str.push('\n');
        }
    }
    (toml_str.trim().to_string(), lines.collect::<Vec<&str>>().join("\n").trim().to_string())
}

pub fn parse_frontmatter(source: &str) -> Result<Meta> {
    let (toml_str, _) = split_meta(source);
    toml::from_str::<Meta>(&toml_str)
        .map_err(|e| anyhow!("failed to parse frontmatter: {}", e))
}