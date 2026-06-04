use std::fs;
use std::path::Path;
use anyhow::Result;

const HILOL_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn init_project(path: &str) -> Result<()> {
    let project_path = Path::new(path);
    
    fs::create_dir_all(project_path.join("pages"))?;
    fs::create_dir_all(project_path.join("templates"))?;
    fs::create_dir_all(project_path.join("static"))?;
    

    let config_content = format!(
        r#"version = "{}"
base_url = "https://example.com"

"#,
        HILOL_VERSION
    );
    fs::write(project_path.join("config.toml"), &config_content)?;
    
    let template_content = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ page.meta.title }}</title>
    <link rel="stylesheet" href="/static/style.css">
</head>
<body>
    <h1>{{ page.meta.title }}</h1>
    {% if page.meta.date %}<p class="date">{{ page.meta.date }}</p>{% endif %}
    {{ page.content | safe }}
</body>
</html>
"#;
    fs::write(project_path.join("templates/default.html"), template_content)?;
    
    // default css
    let css_content = r#"* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: system-ui, -apple-system, sans-serif;
    line-height: 1.6;
    color: #333;
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem;
}

h1, h2, h3 {
    margin: 1.5rem 0 0.5rem;
}

h1 {
    font-size: 2rem;
}

h2 {
    font-size: 1.5rem;
}

p {
    margin-bottom: 1rem;
}

a {
    color: #0066cc;
}

code {
    background: #f4f4f4;
    padding: 0.2rem 0.4rem;
    border-radius: 3px;
    font-family: monospace;
}

pre {
    background: #f4f4f4;
    padding: 1rem;
    border-radius: 3px;
    overflow-x: auto;
}

pre code {
    background: none;
    padding: 0;
}
"#;
    fs::write(project_path.join("static/style.css"), css_content)?;
    
    // Create a sample index page
    let sample_page = r#"---
title = "hilol"
description = "no bs static site generator"
template = "default"
---

this is your first page! edit `pages/index.md` to customize it.

## getting started

1. add more markdown files to the `pages/` directory
2. customize templates in the `templates/` directory
3. add styles to `static/style.css`
4. run `hilol build` to generate your site

## building

```bash
hilol build
```

## serving locally

```bash
hilol serve --reload
```

then open http://localhost:3000 in your browser.

"#;
    fs::write(project_path.join("pages/index.md"), sample_page)?;
    
    println!("created new hilol project at '{}'", path);
    println!("\nNext steps:");
    println!("  cd {}", path);
    println!("  hilol serve --reload");
    
    Ok(())
}
