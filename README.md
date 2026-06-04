# hilol

no bs static site generator

## features

- pretty fast
- small output (its an ssg lol)
- incremental builds
- live reload
- templating using tera
- pretty good errors

## no bs
- no theme system
- no image processing
- no pretty much anything

## to dos

- toc
- tag pages
- rss
- seo stuff
- plugins?

## installation

```bash
cargo install hilol
```

## quick start


```bash
hilol new my-site
cd my-site
```

this generates:
- `config.toml` - site configuration
- `pages/` - content files
- `templates/` - templates
- `static/` - CSS, images, favicon, etc.

### structure

```
my-site/
├── config.toml          
├── pages/               
│   └── index.md
├── templates/           
│   ├── default.html
└── static/              
    ├── style.css
    ├── favicon.svg
    ├── manifest.json
    └── robots.txt
```

## Commands

### build your site

```bash
hilol build [path]
```

renders stuff :)

### serving with live reload

```bash
hilol serve [path] --port 3000 --reload
```

starts a dev server at `http://localhost:3000` with automatic rebuilds on file changes.

### check

```bash
hilol check [path]
```

validates that stuff is okay

### Clean build artifacts

```bash
hilol clean [path]
```

removes `public/` directory and cache files.

## configuration

edit `config.toml`:

```toml
version = "0.1.0"
base_url = "https://example.com"
title = "My Site"
description = "A blazing fast static site"
output_dir = "public"      # default: public
content_dir = "pages"      # default: pages
template_dir = "templates" # default: templates
```

## content files

Markdown files in `pages/` with TOML frontmatter:

```markdown
---
title = "My First Post"
description = "A great post about stuff"
template = "default"
date = "2024-01-01"
---

# My First Post

Your markdown content here...
```

## templates

uses [Tera](https://keats.github.io/tera/) template engine.