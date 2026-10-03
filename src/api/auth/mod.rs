pub mod handlers;
pub mod middleware;
pub mod password;
pub mod rate_limit;

#[cfg(test)]
mod tests;

use axum::routing::{get, post};
use axum::Router;

use crate::state::AppState;
pub use handlers::{
    auth_login_handler, auth_setup_handler, auth_status_handler, auth_verify_handler,
    complete_onboarding_handler, start_onboarding_handler,
};
pub use middleware::{auth_middleware, verify_token_from_parts};
pub use password::change_password_handler;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/status", get(auth_status_handler))
        .route("/setup", post(auth_setup_handler))
        .route("/login", post(auth_login_handler))
        .route("/verify", post(auth_verify_handler))
        .route("/password", post(change_password_handler))
        .route("/onboarding/start", post(start_onboarding_handler))
        .route("/onboarding/complete", post(complete_onboarding_handler))
}
