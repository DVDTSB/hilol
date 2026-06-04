use clap::{Parser, Subcommand};
use std::path::Path;
use anyhow::Result;
mod page;
mod templating;
mod cache;
mod config;
mod build;
mod serve;
mod init;

#[derive(Parser)]
#[command(name = "hilol")]
#[command(about = "no bs static site generator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    New {
        #[arg(help = "Path for the new project")]
        path: String,
    },
    
    Build {
        #[arg(default_value = ".")]
        path: String,
        
        #[arg(short, long)]
        force: bool,
    },
    
    Clean {
        #[arg(default_value = ".")]
        path: String,
    },
    
    Check {
        #[arg(default_value = ".")]
        path: String,
    },
    
    Serve {
        #[arg(default_value = ".")]
        path: String,
        
        #[arg(short, long, default_value = "3000")]
        port: u16,
        
        #[arg(short, long)]
        reload: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("\n:( error: {}", e);
        
        // print error chain for better debugging
        let mut source = e.source();
        while let Some(err) = source {
            eprintln!("  └─ {}", err);
            source = err.source();
        }
        
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::New { path } => {
            init::init_project(&path)?;
        }
        Commands::Build { path, force } => {
            let root = Path::new(&path);
            if force {
                build::clean_site(root)?;
            }
            build::build_site(root)?;
        }
        Commands::Clean { path } => {
            let root = Path::new(&path);
            build::clean_site(root)?;
        }
        Commands::Check { path } => {
            let root = Path::new(&path);
            build::check_site(root)?;
        }
        Commands::Serve { path, port, reload } => {
            let root = Path::new(&path);
            
            // init build
            build::build_site(root)?;
            
            if reload {
                serve::serve_with_reload(root, port).await?;
            } else {
                serve::serve(root, port).await?;
            }
        }
    }

    Ok(())
}
