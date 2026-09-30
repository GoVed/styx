pub mod audit;
pub mod auth;
pub mod chat;
pub mod hitl;
pub mod memory;
pub mod models;
pub mod telemetry;
pub mod tools;
pub mod ws;

use axum::http::{header, HeaderName, Method};
use axum::response::Html;
use axum::routing::get;
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    let cors = if !state.config.allowed_origins.is_empty() {
        let origins: Vec<_> = state
            .config
            .allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                HeaderName::from_static("x-styx-access-key"),
            ])
    } else {
        let default_origins = [
            "http://localhost:3000".parse().unwrap(),
            "http://127.0.0.1:3000".parse().unwrap(),
            "http://localhost:5173".parse().unwrap(),
            "http://127.0.0.1:5173".parse().unwrap(),
        ];
        CorsLayer::new()
            .allow_origin(default_origins)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                HeaderName::from_static("x-styx-access-key"),
            ])
    };

    let static_dir = state.config.static_dir.clone();
    let index_file = static_dir.join("index.html");

    let api_routes = Router::new()
        .nest("/api/auth", auth::router())
        .nest("/api/telemetry", telemetry::router())
        .nest("/api/chat", chat::router())
        .nest("/api/models", models::router())
        .nest("/api/memory", memory::router())
        .nest("/api/tools", tools::router())
        .nest("/api/hitl", hitl::router())
        .nest("/api/audit", audit::router())
        .nest("/ws", ws::router())
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth::auth_middleware,
        ));

    // If static files exist, serve them. If index.html is missing (e.g. before UI build), provide fallback page.
    

    if index_file.exists() {
        api_routes
            .fallback_service(ServeDir::new(&static_dir).fallback(ServeFile::new(index_file)))
            .with_state(state)
            .layer(cors)
    } else {
        api_routes
            .fallback(get(fallback_ui_handler))
            .with_state(state)
            .layer(cors)
    }
}

async fn fallback_ui_handler() -> Html<&'static str> {
    Html(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Styx Mission Control</title>
    <style>
        body { background: #09090b; color: #f4f4f5; font-family: monospace; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; }
        .box { border: 1px solid #27272a; padding: 2rem; border-radius: 8px; max-width: 600px; text-align: center; }
        h1 { color: #10b981; font-size: 1.5rem; margin-bottom: 0.5rem; }
        p { color: #a1a1aa; line-height: 1.6; }
        code { background: #18181b; padding: 0.2rem 0.4rem; border-radius: 4px; color: #38bdf8; }
    </style>
</head>
<body>
    <div class="box">
        <h1>STYX PERSONAL AI OS // HARNESS ACTIVE</h1>
        <p>Backend daemon is fully operational and listening on REST and WebSocket endpoints.</p>
        <p>To compile and mount the Mission Control UI, run:</p>
        <p><code>cd ui && npm install && npm run build</code></p>
    </div>
</body>
</html>"#)
}
