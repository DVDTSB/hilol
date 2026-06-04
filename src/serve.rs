use axum::{Router};
use notify::{Watcher, RecursiveMode, recommended_watcher, EventKind};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tower_http::services::ServeDir;
use anyhow::Result;

pub async fn serve(root_path: &Path, port: u16) -> Result<()> {
    let root_path = root_path.to_path_buf();
    let output_dir = root_path.join("public");
    let static_dir = root_path.join("static");
    
    println!("starting dev server on http://localhost:{}", port);
    println!("serving from: {:?}", output_dir);
    
    let public_dir = ServeDir::new(&output_dir);
    let static_service = ServeDir::new(&static_dir);
    
    let app = Router::new()
        .nest_service("/static", static_service)
        .fallback_service(public_dir);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
        .await?;
    
    println!("server running. press ctrl+c to stop.");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

pub async fn serve_with_reload(root_path: &Path, port: u16) -> Result<()> {
    let root_path = root_path.to_path_buf();
    
    println!("starting dev server with live reload on http://localhost:{}", port);
    println!("watching: {:?}", root_path);
    
    let rebuilding = Arc::new(AtomicBool::new(false));
    let rebuilding_clone = rebuilding.clone();
    let root_for_watcher = root_path.clone();
    let root_clone = root_path.clone();
    
    // File watcher task
    let watcher_task = tokio::spawn(async move {
        if let Ok(mut watcher) = recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                if !matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                    return;
                }
                
                let path_str = event.paths.iter().map(|p| p.to_string_lossy()).collect::<String>();
                if path_str.contains("public") || path_str.contains(".hilol_cache") {
                    return;
                }
                
                if rebuilding_clone.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                    let root = root_clone.clone();
                    let rebuild = rebuilding_clone.clone();
                    tokio::spawn(async move {
                        let _ = crate::build::build_site(&root);
                        rebuild.store(false, Ordering::SeqCst);
                    });
                }
            }
        }) {
            let _ = watcher.watch(root_for_watcher.as_path(), RecursiveMode::Recursive);
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    });

    // Web server
    let public_dir = ServeDir::new(root_path.join("public"));
    let static_dir = ServeDir::new(root_path.join("static"));
    let app = Router::new()
        .nest_service("/static", static_dir)
        .fallback_service(public_dir);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    println!("server running. press ctrl+c to stop.");

    tokio::select! {
        _ = async { axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await } => {},
        _ = watcher_task => {},
    }

    Ok(())
}
