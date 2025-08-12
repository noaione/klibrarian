use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse};

use crate::AppState;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoginForm {
    token: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoginResponse {
    ok: bool,
    error: Option<String>,
}

async fn auth_login(
    State(state): State<AppState>,
    Json(payload): Json<LoginForm>,
) -> impl IntoResponse {
    if state.config.token == payload.token {
        (
            StatusCode::OK,
            Json(LoginResponse {
                ok: true,
                error: None,
            }),
        )
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(LoginResponse {
                ok: false,
                error: Some("Invalid token".to_string()),
            }),
        )
    }
}

pub fn auth_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/login", axum::routing::post(auth_login))
        .with_state(state)
}
