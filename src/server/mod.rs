mod state;
mod routes;
#[cfg(test)]
mod routes_test;

use std::path::PathBuf;

use anyhow::Result;
use axum::Router;
use tower_http::cors::CorsLayer;

use state::AppState;

pub async fn run_server(workspace: PathBuf, host: &str, port: u16, static_dir: Option<PathBuf>) -> Result<()> {
    println!("Workspace: {}", workspace.display());
    let state = AppState::new(workspace);

    let api_routes = routes::api_router();

    let static_path = static_dir
        .or_else(|| {
            let builtin = PathBuf::from("/usr/share/ai-arch-story/webapp");
            if builtin.exists() { Some(builtin) } else { None }
        });

    let mut app = Router::new()
        .nest("/api/v1", api_routes)
        .layer(CorsLayer::permissive())
        .with_state(state);

    if let Some(dir) = static_path {
        println!("Serving webapp from: {}", dir.display());
        app = app.fallback_service(tower_http::services::ServeDir::new(dir));
    }

    let addr = format!("{}:{}", host, port);
    println!("Serving on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
