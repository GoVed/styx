pub mod discovery;
pub mod inbound;
pub mod inspect;
pub mod servers;

use axum::routing::{delete, get, post};
use axum::Router;

use crate::state::AppState;
pub use discovery::{discover_local_tools, install_tool};
pub use inbound::handle_inbound_tool_event;
pub use inspect::inspect_tool;
pub use servers::{
    add_server, list_servers, list_tools, remove_server, test_call_tool, update_policy,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/servers", get(list_servers).post(add_server))
        .route("/servers/{id}", delete(remove_server))
        .route("/list", get(list_tools))
        .route("/policy", post(update_policy))
        .route("/call", post(test_call_tool))
        .route("/trigger", post(handle_inbound_tool_event))
        .route("/discover", get(discover_local_tools))
        .route("/inspect", post(inspect_tool))
        .route("/install", post(install_tool))
}
