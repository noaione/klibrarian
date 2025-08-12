use axum::{
    Json,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::AppState;

pub async fn auth_middleware(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let token = state.config.token.clone();
    let headers: &HeaderMap = req.headers();

    if let Some(auth_header) = headers.get(axum::http::header::AUTHORIZATION) {
        if let Ok(auth_value) = auth_header.to_str() {
            // Check if starts with "Bearer "
            if let Some(token_value) = auth_value.strip_prefix("Bearer ") {
                // Skip "Bearer "
                if token_value == token {
                    next.run(req).await
                } else {
                    (
                        StatusCode::UNAUTHORIZED,
                        Json(serde_json::json!({
                            "ok": false,
                            "error": "Invalid token"
                        })),
                    )
                        .into_response()
                }
            } else {
                // reject if it doesn't start with "Bearer "
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "ok": false,
                        "error": "Invalid authorization format. Expected 'Bearer <token>'"
                    })),
                )
                    .into_response()
            }
        } else {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "ok": false,
                    "error": "Invalid authorization header format"
                })),
            )
                .into_response()
        }
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "ok": false,
                "error": "Unauthorized access. No authorization header provided."
            })),
        )
            .into_response()
    }
}
