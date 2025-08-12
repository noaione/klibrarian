use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use garde::Validate;
use serde_json::Value;
use tracing::{error, info};

use crate::{
    AppState,
    database::{InviteToken, KomgaInviteOption, NavidromeInviteOption},
    invitee::{InviteTokenApplicationPayload, create_user_in},
    routes::middleware::auth_middleware,
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct InviteQuery {
    token: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind")]
pub enum InviteRequestParams {
    #[serde(rename = "komga")]
    Komga(KomgaInviteOption),
    #[serde(rename = "navidrome")]
    Navidrome(NavidromeInviteOption),
}

pub async fn create_invite_token(
    State(state): State<AppState>,
    Json(option): Json<InviteRequestParams>,
) -> impl IntoResponse {
    let generated_token = match option {
        InviteRequestParams::Komga(komga_option) => InviteToken::create_komga(komga_option),
        InviteRequestParams::Navidrome(navidrome_option) => {
            InviteToken::create_navidrome(navidrome_option)
        }
    };

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());

    // store the token in SQL
    match state.db.add_invite(&generated_token).await {
        Ok(_) => {
            let invite_token_value = serde_json::to_value(&generated_token).unwrap();

            // wrap the json in a {"ok": true, "data": {}} object
            let wrapped_json: Value = serde_json::json!({
                "ok": true,
                "data": invite_token_value
            });

            (
                StatusCode::OK,
                headers,
                serde_json::to_string(&wrapped_json).unwrap(),
            )
        }
        Err(error) => {
            // wrap the json in a {"ok": true, "data": {}} object
            let wrapped_json: Value = serde_json::json!({
                "ok": false,
                "error": format!("Failed to create invite token: {}", error)
            });

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                headers,
                serde_json::to_string(&wrapped_json).unwrap(),
            )
        }
    }
}

pub async fn get_invite_config(State(state): State<AppState>) -> impl IntoResponse {
    // Get all the options available in Komga
    let labels = match state.komga.get_sharing_labels().await {
        Ok(labels) => labels,
        Err(_) => {
            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", "application/json".parse().unwrap());

            // wrap the json in a {"ok": true, "data": {}} object
            let wrapped_json: Value = serde_json::json!({
                "ok": false,
                "error": "Failed to get labels from Komga"
            });

            return (
                StatusCode::SERVICE_UNAVAILABLE,
                headers,
                serde_json::to_string(&wrapped_json).unwrap(),
            );
        }
    };
    let libraries = match state.komga.get_libraries().await {
        Ok(libraries) => libraries,
        Err(_) => {
            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", "application/json".parse().unwrap());

            // wrap the json in a {"ok": true, "data": {}} object
            let wrapped_json: Value = serde_json::json!({
                "ok": false,
                "error": "Failed to get libraries from Komga"
            });

            return (
                StatusCode::SERVICE_UNAVAILABLE,
                headers,
                serde_json::to_string(&wrapped_json).unwrap(),
            );
        }
    };

    let navidrome_libraries = match &state.navidrome {
        Some(navidrome) => {
            let mut client = navidrome.lock().await;
            match client.get_library().await {
                Ok(libraries) => {
                    drop(client); // release the lock early
                    Some(libraries)
                }
                Err(_) => {
                    drop(client); // release the lock early
                    let mut headers = HeaderMap::new();
                    headers.insert("Content-Type", "application/json".parse().unwrap());

                    // wrap the json in a {"ok": true, "data": {}} object
                    let wrapped_json: Value = serde_json::json!({
                        "ok": false,
                        "error": "Failed to get libraries from Navidrome"
                    });

                    return (
                        StatusCode::SERVICE_UNAVAILABLE,
                        headers,
                        serde_json::to_string(&wrapped_json).unwrap(),
                    );
                }
            }
        }
        None => None,
    };

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());

    // wrap the json in a {"ok": true, "data": {}} object
    let wrapped_json: Value = serde_json::json!({
        "ok": true,
        "data": {
            "komga": {
                "active": true,
                "labels": labels,
                "libraries": libraries
            },
            "navidrome": {
                "active": navidrome_libraries.is_some(),
                "libraries": navidrome_libraries.unwrap_or_default()
            },
        }
    });

    (
        StatusCode::OK,
        headers,
        serde_json::to_string(&wrapped_json).unwrap(),
    )
}

pub async fn get_invite_token(
    State(state): State<AppState>,
    Path(token): Path<uuid::Uuid>,
) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());

    match state.db.get_invite(token).await {
        Ok(Some(data)) => {
            // check if the token is expired
            if data.is_expired() {
                match state.db.delete_invite(token).await {
                    Ok(_) => {
                        // wrap the json in a {"ok": true, "data": {}} object
                        let wrapped_json: Value = serde_json::json!({
                            "ok": false,
                            "error": "Invite token expired"
                        });

                        (
                            StatusCode::FORBIDDEN,
                            headers,
                            serde_json::to_string(&wrapped_json).unwrap(),
                        )
                    }
                    Err(error) => {
                        error!("Failed to delete expired invite token: {}", error);

                        let wrapped_json: Value = serde_json::json!({
                            "ok": false,
                            "error": format!("Failed to delete expired invite token: {}", token)
                        });

                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            headers,
                            serde_json::to_string(&wrapped_json).unwrap(),
                        )
                    }
                }
            } else {
                // wrap the json in a {"ok": true, "data": {}} object
                let wrapped_json: Value = serde_json::json!({
                    "ok": true,
                    "data": data,
                });

                (
                    StatusCode::OK,
                    headers,
                    serde_json::to_string(&wrapped_json).unwrap(),
                )
            }
        }
        Ok(None) => {
            // wrap the json in a {"ok": true, "data": {}} object
            let wrapped_json: Value = serde_json::json!({
                "ok": false,
                "error": "Invite token not found"
            });

            (
                StatusCode::NOT_FOUND,
                headers,
                serde_json::to_string(&wrapped_json).unwrap(),
            )
        }
        Err(error) => {
            // wrap the json in a {"ok": true, "data": {}} object
            let wrapped_json: Value = serde_json::json!({
                "ok": false,
                "error": format!("Failed to get invite token: {}", error)
            });

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                headers,
                serde_json::to_string(&wrapped_json).unwrap(),
            )
        }
    }
}

pub async fn delete_invite_token(
    State(state): State<AppState>,
    Path(token): Path<uuid::Uuid>,
) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());

    match state.db.delete_invite(token).await {
        Ok(_) => {
            // wrap the json in a {"ok": true, "data": {}} object
            let wrapped_json: Value = serde_json::json!({
                "ok": true,
            });
            (
                StatusCode::OK,
                headers,
                serde_json::to_string(&wrapped_json).unwrap(),
            )
        }
        Err(error) => {
            // wrap the json in a {"ok": true, "data": {}} object
            let wrapped_json: Value = serde_json::json!({
                "ok": false,
                "error": format!("Failed to delete invite token: {}", error)
            });
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                headers,
                serde_json::to_string(&wrapped_json).unwrap(),
            )
        }
    }
}

pub async fn apply_invite_token(
    State(state): State<AppState>,
    Path(token): Path<uuid::Uuid>,
    Json(request): Json<InviteTokenApplicationPayload>,
) -> impl IntoResponse {
    if let Err(e) = request.validate() {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());

        let mut format_err = String::new();
        for (field, err) in e.iter() {
            format_err.push_str(&format!("- {field}: {err}"));
            format_err.push('\n');
        }

        // wrap the json in a {"ok": true, "data": {}} object
        let wrapped_json: Value = serde_json::json!({
            "ok": false,
            "error": format!("Invalid request:\n{}", format_err)
        });

        return (
            StatusCode::BAD_REQUEST,
            headers,
            serde_json::to_string(&wrapped_json).unwrap(),
        );
    }

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());

    info!("Applying invite token: {}", token);
    match state.db.get_invite(token).await {
        Ok(Some(data)) => {
            if data.is_expired() {
                // wrap the json in a {"ok": true, "data": {}} object
                let wrapped_json: Value = serde_json::json!({
                    "ok": false,
                    "error": "Invite token expired"
                });

                return (
                    StatusCode::FORBIDDEN,
                    headers,
                    serde_json::to_string(&wrapped_json).unwrap(),
                );
            }

            // Create user in Komga
            match create_user_in(&state, &data, &request).await {
                Ok(target_host) => {
                    // wrap the json in a {"ok": true, "data": {}} object
                    let wrapped_json: Value = serde_json::json!({
                        "ok": true,
                        "data": {
                            "host": target_host,
                        }
                    });
                    (
                        StatusCode::OK,
                        headers,
                        serde_json::to_string(&wrapped_json).unwrap(),
                    )
                }
                Err(e) => {
                    error!("Failed to create user in Komga: {}", e);
                    let wrapped_json: Value = serde_json::json!({
                        "ok": false,
                        "error": format!("Failed to create user: {}", e)
                    });
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        headers,
                        serde_json::to_string(&wrapped_json).unwrap(),
                    )
                }
            }
        }
        Ok(None) => {
            // wrap the json in a {"ok": true, "data": {}} object
            let wrapped_json: Value = serde_json::json!({
                "ok": false,
                "error": "Invite token not found"
            });

            (
                StatusCode::NOT_FOUND,
                headers,
                serde_json::to_string(&wrapped_json).unwrap(),
            )
        }
        Err(e) => {
            error!("Failed to get invite token: {}", e);
            // wrap the json in a {"ok": true, "data": {}} object
            let wrapped_json: Value = serde_json::json!({
                "ok": false,
                "error": format!("Failed to get invite token: {}", e)
            });
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                headers,
                serde_json::to_string(&wrapped_json).unwrap(),
            )
        }
    }
}

pub async fn get_all_invite_token(State(state): State<AppState>) -> impl IntoResponse {
    let tokens = state.db.get_all_invites().await;
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());

    match tokens {
        Ok(tokens) => {
            // wrap the json in a {"ok": true, "data": {}} object
            let wrapped_json: Value = serde_json::json!({
                "ok": true,
                "data": tokens,
            });

            (
                StatusCode::OK,
                headers,
                serde_json::to_string(&wrapped_json).unwrap(),
            )
        }
        Err(error) => {
            // wrap the json in a {"ok": true, "data": {}} object
            tracing::error!("Failed to get invite tokens: {:?}", error);
            let wrapped_json: Value = serde_json::json!({
                "ok": false,
                "error": format!("Failed to get invite tokens: {}", error)
            });

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                headers,
                serde_json::to_string(&wrapped_json).unwrap(),
            )
        }
    }
}

pub async fn get_info(State(state): State<AppState>) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());

    let mut active_servers = vec!["komga"];
    if state.navidrome.is_some() {
        active_servers.push("navidrome");
    }
    let wrapped_json = serde_json::json!({
        "ok": true,
        "data": {
            "servers": active_servers,
            "v": env!("CARGO_PKG_VERSION"),
        }
    });

    (StatusCode::OK, Json(wrapped_json))
}

pub fn invite_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            axum::routing::get(get_all_invite_token).post(create_invite_token),
        )
        .route(
            "/{token}",
            axum::routing::get(get_invite_token).delete(delete_invite_token),
        )
        .route("/{token}/apply", axum::routing::post(apply_invite_token))
        .route("/config", axum::routing::get(get_invite_config))
        .route("/info", axum::routing::get(get_info))
        .with_state(state.clone())
        .layer(axum::middleware::from_fn_with_state(state, auth_middleware))
}
