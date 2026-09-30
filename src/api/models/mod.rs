pub mod activate;
pub mod configs;
pub mod containers;
pub mod presets;

use axum::routing::{delete, get, post};
use axum::Router;

use crate::state::AppState;
pub use activate::activate_container_as_model;
pub use configs::{
    activate_model_config, add_model_config, auto_sync_active_local_model, delete_model_config,
    list_model_configs, test_connection,
};
pub use containers::{
    delete_container, deploy_container, get_container_logs,
    list_containers, restart_container, start_container, stop_container,
};
pub use presets::{
    get_quantization_recommendation, list_local_model_files, list_presets, preview_docker_command,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/containers", get(list_containers))
        .route("/deploy", post(deploy_container))
        .route("/containers/{id}/start", post(start_container))
        .route("/containers/{id}/stop", post(stop_container))
        .route("/containers/{id}/restart", post(restart_container))
        .route("/containers/{id}", delete(delete_container))
        .route("/containers/{id}/activate", post(activate_container_as_model))
        .route("/containers/{id}/logs", get(get_container_logs))
        .route("/presets", get(list_presets))
        .route("/local-files", get(list_local_model_files))
        .route("/recommend-quantization", get(get_quantization_recommendation))
        .route("/preview-command", post(preview_docker_command))
        .route("/configs", get(list_model_configs).post(add_model_config))
        .route("/configs/{id}/activate", post(activate_model_config))
        .route("/configs/{id}", delete(delete_model_config))
        .route("/test-connection", post(test_connection))
}
