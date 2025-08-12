use axum::Router;

use crate::AppState;

pub mod auth;
pub mod invite;
pub(super) mod middleware;

pub fn api(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::auth_routes(state.clone()))
        .nest("/invite", invite::invite_routes(state.clone()))
        .with_state(state.clone())
}
