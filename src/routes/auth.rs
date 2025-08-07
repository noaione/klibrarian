use axum::{Json, Router, http::StatusCode, response::IntoResponse};

use crate::AppState;

use super::AuthToken;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoginForm {
    token: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoginResponse {
    ok: bool,
    error: Option<String>,
}

async fn auth_login(Json(payload): Json<LoginForm>) -> impl IntoResponse {
    let token = std::env::var("TOKEN");

    match token {
        Ok(token) => {
            if token == payload.token {
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
        Err(_) => (
            StatusCode::UNAUTHORIZED,
            Json(LoginResponse {
                ok: false,
                error: Some("Invalid token".to_string()),
            }),
        ),
    }
}

async fn auth_test(AuthToken(_): AuthToken) -> impl IntoResponse {
    Json(LoginResponse {
        ok: true,
        error: None,
    })
}

pub fn auth_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/login", axum::routing::post(auth_login))
        .route("/test", axum::routing::get(auth_test))
        .with_state(state)
}
