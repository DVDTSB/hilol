use serde::Serialize;
use tera::{Tera, Context};
use crate::page::Page;
use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use anyhow::{Result, anyhow};

#[derive(Debug, Serialize)]
pub struct SiteContext {
    pub pages: Vec<Page>,
    pub base_url: String,
}

pub fn render_page(page: &Page, site_context: &SiteContext, template_dir: &Path) -> Result<String> {
    let mut tera = Tera::default();
    
    // load all templates
    for entry in WalkDir::new(template_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "html"))
    {
        let file_path = entry.path();
        let template_name = file_path
            .strip_prefix(template_dir)?
            .to_string_lossy()
            .replace("\\", "/");
        
        let content = fs::read_to_string(&file_path)?;
        tera.add_raw_template(&template_name, &content)?;
    }
    
    // Get template name, defaulting to "default" and adding .html if not present
    let mut template_name = page.meta.template.as_deref().unwrap_or("default").to_string();
    if !template_name.ends_with(".html") {
        template_name.push_str(".html");
    }
    
    let mut context = Context::new();
    context.insert("page", page);
    context.insert("site", site_context);
    
    tera.render(&template_name, &context)
        .map_err(|e| anyhow!("failed to render '{}': {}", template_name, e))
}