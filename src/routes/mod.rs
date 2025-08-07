use axum::{
    Router,
    extract::FromRequestParts,
    http::{HeaderMap, HeaderValue, StatusCode, request::Parts},
    response::IntoResponse,
};

use crate::AppState;

pub mod auth;
pub mod invite;

pub fn api(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::auth_routes(state.clone()))
        .nest("/invite", invite::invite_routes(state.clone()))
        .with_state(state.clone())
}

#[allow(dead_code)]
pub struct AuthToken(HeaderValue);

#[derive(serde::Serialize)]
pub struct RejectAuthToken {
    ok: bool,
    error: &'static str,
}

impl<S> FromRequestParts<S> for AuthToken
where
    S: Send + Sync,
{
    type Rejection = RejectAuthToken;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        if let Some(auth_header) = parts.headers.get("Authorization") {
            let auth_str = auth_header.to_str().unwrap();

            if auth_str.starts_with("Bearer ") {
                let token = auth_str.replace("Bearer ", "");
                // Verify with env

                if token != std::env::var("TOKEN").unwrap() {
                    return Err(RejectAuthToken {
                        ok: false,
                        error: "Invalid token",
                    });
                }

                match HeaderValue::from_str(&token) {
                    Ok(token) => Ok(AuthToken(token)),
                    Err(_) => Err(RejectAuthToken {
                        ok: false,
                        error: "Invalid token format",
                    }),
                }
            } else {
                Err(RejectAuthToken {
                    ok: false,
                    error: "Missing Bearer prefix",
                })
            }
        } else {
            Err(RejectAuthToken {
                ok: false,
                error: "Missing Authorization header",
            })
        }
    }
}

impl IntoResponse for RejectAuthToken {
    fn into_response(self) -> axum::response::Response {
        // build response
        let response = serde_json::to_string(&self).unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        (StatusCode::UNAUTHORIZED, headers, response).into_response()
    }
}
