use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/tree", get(list_tree))
        .route("/file", get(read_file).post(write_file).delete(delete_file))
        .route("/create", post(create_file))
        .route("/search", get(search_memory))
}

#[derive(Deserialize)]
struct FileQuery {
    path: String,
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    limit: Option<usize>,
}

#[derive(Deserialize)]
struct WriteFileRequest {
    path: String,
    content: String,
    section: Option<String>,
}

#[derive(Deserialize)]
struct CreateFileRequest {
    category: String,
    filename: String,
    content: String,
}

async fn list_tree(State(state): State<AppState>) -> impl IntoResponse {
    match state.memory.list_tree() {
        Ok(tree) => (StatusCode::OK, Json(json!({ "success": true, "files": tree }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}

async fn read_file(
    State(state): State<AppState>,
    Query(query): Query<FileQuery>,
) -> impl IntoResponse {
    match state.memory.read_file(&query.path) {
        Ok(content) => (
            StatusCode::OK,
            Json(json!({ "success": true, "path": query.path, "content": content })),
        ),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}

async fn write_file(
    State(state): State<AppState>,
    Json(payload): Json<WriteFileRequest>,
) -> impl IntoResponse {
    match state
        .memory
        .write_file(&payload.path, &payload.content, payload.section.as_deref())
        .await
    {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({ "success": true, "path": payload.path })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}

async fn create_file(
    State(state): State<AppState>,
    Json(payload): Json<CreateFileRequest>,
) -> impl IntoResponse {
    match state
        .memory
        .create_file(&payload.category, &payload.filename, &payload.content)
        .await
    {
        Ok(path) => (
            StatusCode::CREATED,
            Json(json!({ "success": true, "path": path })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}

async fn delete_file(
    State(state): State<AppState>,
    Query(query): Query<FileQuery>,
) -> impl IntoResponse {
    match state.memory.delete_file(&query.path).await {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({ "success": true, "path": query.path })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}

async fn search_memory(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(10);
    match state.memory.search(&query.q, limit) {
        Ok(results) => (
            StatusCode::OK,
            Json(json!({ "success": true, "results": results })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}
