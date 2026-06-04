use crate::page::Page;
use crate::templating::SiteContext;
use crate::{cache, config};
use anyhow::Result;
use pulldown_cmark::{html, Parser, Options};
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;
use walkdir::WalkDir;

pub struct PageWithHash {
    pub page: Page,
    pub content_hash: String,
    pub template_hash: String,
}

pub fn build_site(root_path: &Path) -> Result<()> {
    let build_start = Instant::now(); //for seeing how fast it is :3

    let config_path = root_path.join("config.toml");
    let pages_path = root_path.join("pages");
    let templates_path = root_path.join("templates");
    let output_path = root_path.join("public");
    let cache_path = root_path.join(".hilol_cache.json");

    // Validate required directories exist
    if !pages_path.exists() {
        return Err(anyhow::anyhow!("pages directory not found at {:?}. Run 'hilol new' to create a project.", pages_path));
    }
    if !templates_path.exists() {
        return Err(anyhow::anyhow!("templates directory not found at {:?}", templates_path));
    }

    println!("reading config from {:?}", config_path);
    let config = config::Config::load(&config_path)?;

    println!("output directory: {:?}", output_path);
    fs::create_dir_all(&output_path)?;

    let cache = Mutex::new(cache::Cache::load(&cache_path)?);

    println!("scanning pages directory...");
    let mut pages_with_hash = Vec::new();
    let mut parse_errors = Vec::new();

    for entry in WalkDir::new(&pages_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
    {
        let file_path = entry.path();

        // hash the content file
        let content = fs::read_to_string(&file_path)?;
        let content_hash = cache::hash_string(&content);

        let file_name = file_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| {
                eprintln!("warning: skipping file with invalid name: {:?}", file_path);
                String::new()
            });

        if file_name.is_empty() {
            continue;
        }

        // parse page
        match crate::page::parse(&file_name, &content) {
            Ok(mut page) => {
                // convert markdown to html
                let parser = Parser::new_ext(&page.content, Options::all());
                let mut html = String::new();
                html::push_html(&mut html, parser);
                
                // clean up extra whitespace in code blocks (remove leading newlines after <code>)
                html = html.replace("<code>\n", "<code>");
                
                page.content = html;

                // compute hash of the template this page uses
                let mut template_name = page.meta.template.as_deref().unwrap_or("default.html").to_string();
                // if the template name doesnt end with .html
                if !template_name.ends_with(".html") {
                    template_name.push_str(".html");
                }

                let template_path = templates_path.join(&template_name);
                let template_hash = cache::hash_file(&template_path).unwrap_or_else(|e| {
                    eprintln!("warning: failed to hash template {}: {}", template_name, e);
                    String::new()
                });

                pages_with_hash.push(PageWithHash {
                    page,
                    content_hash,
                    template_hash,
                });
            }
            Err(e) => {
                parse_errors.push(format!("  {} ({})", file_path.display(), e));
            }
        }
    }

    // report parse errors if any
    if !parse_errors.is_empty() {
        println!("\nparse errors:");
        for error in &parse_errors {
            println!("{}", error);
        }
        return Err(anyhow::anyhow!("{} file(s) had parse errors", parse_errors.len()));
    }

    println!("parsed {} pages", pages_with_hash.len());

    // create site context
    let pages: Vec<Page> = pages_with_hash.iter().map(|p| p.page.clone()).collect();
    let site_context = SiteContext {
        pages,
        base_url: config.base_url.clone(),
    };

    // render pages
    println!("rendering pages...");
    let mut all_errors = Vec::new();
    let mut rendered = 0;
    let mut skipped = 0;

    for page_hash in &pages_with_hash {
        let page_key = format!("pages/{}.md", page_hash.page.slug);
        
        // skip draft pages
        if page_hash.page.meta.draft == Some(true) {
            skipped += 1;
            continue;
        }
        
        if !cache.lock().unwrap().is_dirty(&page_key, &page_hash.content_hash, &page_hash.template_hash) {
            skipped += 1;
            continue;
        }

        match crate::templating::render_page(&page_hash.page, &site_context, &templates_path) {
            Ok(html) => {
                let output_file = output_path.join(format!("{}.html", page_hash.page.slug));
                if let Err(e) = fs::write(&output_file, html) {
                    all_errors.push(format!("  {} ({})", output_file.display(), e));
                } else {
                    rendered += 1;
                    cache.lock().unwrap().update(
                        page_key,
                        page_hash.content_hash.clone(),
                        page_hash.template_hash.clone(),
                    );
                }
            }
            Err(e) => all_errors.push(format!("  {} ({})", page_hash.page.slug, e)),
        }
    }

    if rendered > 0 {
        println!("  rendered {} page(s)", rendered);
    }
    if skipped > 0 {
        println!("  skipped {} page(s) (unchanged)", skipped);
    }
    
    if !all_errors.is_empty() {
        println!("\nerrors:");
        for error in &all_errors {
            println!("{}", error);
        }
        return Err(anyhow::anyhow!("{} error(s) during rendering", all_errors.len()));
    }

    // save cache
    let final_cache = cache.lock().unwrap();
    final_cache.save(&cache_path)?;
    let elapsed = build_start.elapsed();
    println!("build complete! ({:.2}s)", elapsed.as_secs_f64());
    Ok(())
}

pub fn clean_site(root_path: &Path) -> Result<()> {
    let output_path = root_path.join("public");
    let cache_path = root_path.join(".hilol_cache.json");

    if output_path.exists() {
        fs::remove_dir_all(&output_path)?;
        println!("cleaned output directory: {:?}", output_path);
    }

    if cache_path.exists() {
        fs::remove_file(&cache_path)?;
        println!("cleared cache");
    }

    println!("clean complete!");
    Ok(())
}

pub fn check_site(root_path: &Path) -> Result<()> {
    let config_path = root_path.join("config.toml");
    let pages_path = root_path.join("pages");
    let templates_path = root_path.join("templates");

    println!("checking site configuration...");

    // check config
    if !config_path.exists() {
        return Err(anyhow::anyhow!("config.toml not found"));
    }
    let _config = config::Config::load(&config_path)?;
    println!("  config.toml valid");

    // check templates
    if !templates_path.exists() {
        return Err(anyhow::anyhow!("templates directory not found"));
    }
    let default_template = templates_path.join("default.html");
    if !default_template.exists() {
        return Err(anyhow::anyhow!("templates/default.html not found"));
    }
    println!("  templates directory found");

    // check pages
    if !pages_path.exists() {
        return Err(anyhow::anyhow!("pages directory not found"));
    }

    let mut page_count = 0;
    let mut check_errors = Vec::new();
    
    for entry in WalkDir::new(&pages_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
    {
        let file_path = entry.path();
        let content = fs::read_to_string(&file_path)?;

        // validate frontmatter
        match crate::page::parse_frontmatter(&content) {
            Ok(_) => {
                page_count += 1;
                println!("  {:?}", file_path.file_name().unwrap());
            }
            Err(e) => {
                check_errors.push(format!("  error! {:?}: {}", file_path.file_name().unwrap(), e));
            }
        }
    }

    if !check_errors.is_empty() {
        for error in check_errors {
            println!("{}", error);
        }
    }

    println!("checking complete! {} pages found", page_count);
    Ok(())
}
