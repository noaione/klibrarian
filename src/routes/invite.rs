use std::collections::HashMap;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use garde::Validate;
use redis::{AsyncCommands, aio::MultiplexedConnection};
use serde_json::Value;
use tracing::{error, info};

use crate::{
    AppState,
    komga::{
        KomgaClient, KomgaUserCreate, KomgaUserCreateOption, KomgaUserCreateOptionSharedLibraries,
    },
};

use super::AuthToken;

const KLIBRARIAN_INVITE_TOKEN: &str = "k-librarian:invite_tokens";
const DEFAULT_ROLES: &[&str] = &["USER", "FILE_DOWNLOAD", "PAGE_STREAMING"];

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct InviteOption {
    #[serde(rename = "labelsAllow")]
    pub labels_allow: Option<Vec<String>>,
    #[serde(rename = "labelsExclude")]
    pub labels_exclude: Option<Vec<String>>,
    #[serde(rename = "sharedLibraries")]
    pub shared_libraries: Option<KomgaUserCreateOptionSharedLibraries>,
    #[serde(rename = "expiresAt")]
    pub expire_at: Option<u64>,
    #[serde(rename = "roles")]
    pub roles: Option<Vec<String>>,
}

impl From<InviteOption> for KomgaUserCreateOption {
    fn from(val: InviteOption) -> Self {
        KomgaUserCreateOption {
            labels_allow: val.labels_allow,
            labels_exclude: val.labels_exclude,
            shared_libraries: val.shared_libraries,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct InviteToken {
    token: String,
    option: InviteOption,
    user_id: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, garde::Validate)]
pub struct InviteTokenApplicationRequest {
    #[garde(email)]
    email: String,
    #[garde(length(min = 6))]
    password: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct InviteQuery {
    token: String,
}

pub async fn create_invite_token(
    State(state): State<AppState>,
    _: AuthToken,
    Json(option): Json<InviteOption>,
) -> impl IntoResponse {
    let token = uuid::Uuid::new_v4().to_string();

    let invite_token = InviteToken {
        token: token.clone(),
        user_id: None,
        option,
    };

    let mut redis_conn = state
        .redis
        .get_multiplexed_async_connection()
        .await
        .unwrap();
    // use sets to store our tokens
    let res: Result<i32, redis::RedisError> = redis_conn
        .hset(
            KLIBRARIAN_INVITE_TOKEN,
            token.clone(),
            serde_json::to_string(&invite_token).unwrap(),
        )
        .await;

    match res {
        Ok(_) => {}
        Err(error) => {
            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", "application/json".parse().unwrap());

            // wrap the json in a {"ok": true, "data": {}} object
            let wrapped_json: Value = serde_json::json!({
                "ok": false,
                "error": format!("Failed to create invite token: {}", error)
            });

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                headers,
                serde_json::to_string(&wrapped_json).unwrap(),
            );
        }
    }

    let invite_token_json: Value =
        serde_json::from_str(&serde_json::to_string(&invite_token).unwrap()).unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());

    // wrap the json in a {"ok": true, "data": {}} object
    let wrapped_json: Value = serde_json::json!({
        "ok": true,
        "data": invite_token_json
    });

    (
        StatusCode::OK,
        headers,
        serde_json::to_string(&wrapped_json).unwrap(),
    )
}

pub async fn get_invite_config(_: AuthToken) -> impl IntoResponse {
    // Get all the options available in Komga

    let komga = KomgaClient::instance();

    let labels = match komga.get_sharing_labels().await {
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
    let libraries = match komga.get_libraries().await {
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

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());

    // wrap the json in a {"ok": true, "data": {}} object
    let wrapped_json: Value = serde_json::json!({
        "ok": true,
        "data": {
            "labels": labels,
            "libraries": libraries
        }
    });

    (
        StatusCode::OK,
        headers,
        serde_json::to_string(&wrapped_json).unwrap(),
    )
}

async fn remove_token_or(
    redis_conn: &mut MultiplexedConnection,
    token: &InviteToken,
) -> Result<(), ()> {
    let current_unix: u64 = chrono::Utc::now().timestamp() as u64;

    match token.option.expire_at {
        Some(expire_at) => {
            if current_unix > expire_at {
                redis_conn
                    .hdel(KLIBRARIAN_INVITE_TOKEN, token.token.clone())
                    .await
                    .unwrap_or(0);
                Err(())
            } else {
                Ok(())
            }
        }
        None => Ok(()),
    }
}

pub async fn get_invite_token(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> impl IntoResponse {
    let mut redis_conn = state
        .redis
        .get_multiplexed_async_connection()
        .await
        .unwrap();

    let data: Result<String, _> = redis_conn
        .hget(KLIBRARIAN_INVITE_TOKEN, token.clone())
        .await;

    match data {
        Ok(data) => {
            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", "application/json".parse().unwrap());

            let raw_val: InviteToken = serde_json::from_str(&data).unwrap();

            match remove_token_or(&mut redis_conn, &raw_val).await {
                Ok(_) => {
                    // wrap the json in a {"ok": true, "data": {}} object
                    let wrapped_json: Value = serde_json::json!({
                        "ok": true,
                        "data": raw_val,
                    });

                    (
                        StatusCode::OK,
                        headers,
                        serde_json::to_string(&wrapped_json).unwrap(),
                    )
                }
                Err(_) => {
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
            }
        }
        Err(_) => {
            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", "application/json".parse().unwrap());

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
    }
}

pub async fn delete_invite_token(
    _: AuthToken,
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> impl IntoResponse {
    let mut redis_conn = state
        .redis
        .get_multiplexed_async_connection()
        .await
        .unwrap();

    let data = redis_conn
        .hdel(KLIBRARIAN_INVITE_TOKEN, token)
        .await
        .unwrap_or(0);

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());

    let ok = data > 0;

    // wrap the json in a {"ok": true, "data": {}} object
    let wrapped_json: Value = serde_json::json!({
        "ok": ok,
    });

    (
        StatusCode::OK,
        headers,
        serde_json::to_string(&wrapped_json).unwrap(),
    )
}

pub async fn create_user_in_komga(
    redis_conn: &mut MultiplexedConnection,
    komga: &crate::komga::KomgaClient,
    token: &InviteToken,
    payload: &InviteTokenApplicationRequest,
) -> Result<(), anyhow::Error> {
    let roles = token.option.roles.clone().unwrap_or(
        DEFAULT_ROLES
            .to_vec()
            .iter()
            .map(|x| x.to_string())
            .collect(),
    );

    if let Some(user_id) = token.user_id.clone() {
        info!(
            "[{} / {}] User already created, applying restriction",
            token.token, user_id
        );
        // do apply user restriction
        let resp_restrict = komga
            .apply_user_restriction(user_id, token.option.clone().into())
            .await;

        match resp_restrict {
            Ok(_) => {
                // remove the token
                redis_conn
                    .hdel(KLIBRARIAN_INVITE_TOKEN, token.token.clone())
                    .await
                    .unwrap_or(0);

                return Ok(());
            }
            Err(error) => {
                error!(
                    "[{}] Failed applying restriction... ({})",
                    token.token, error
                );
                anyhow::bail!("Failed to apply user restriction")
            }
        }
    }

    let user_create = KomgaUserCreate {
        email: payload.email.clone(),
        password: payload.password.clone(),
        roles,
    };

    info!("[{}] Creating user...", token.token);
    let res = komga.create_user(user_create).await;

    match res {
        Ok(data) => {
            // save the user id
            let invite_token = InviteToken {
                token: token.token.clone(),
                option: token.option.clone(),
                user_id: Some(data.id.clone()),
            };

            info!(
                "[{}] Done creating user, saving temp user ID... ({})",
                token.token,
                data.id.clone()
            );
            let _ = redis_conn
                .hset(
                    KLIBRARIAN_INVITE_TOKEN,
                    token.token.clone(),
                    serde_json::to_string(&invite_token).unwrap(),
                )
                .await
                .unwrap_or(0);

            // do user restriction
            info!(
                "[{}] Applying restriction for... ({})",
                token.token,
                data.id.clone()
            );
            let resp_restrict = komga
                .apply_user_restriction(data.id.clone(), token.option.clone().into())
                .await;

            match resp_restrict {
                Ok(_) => {
                    // remove the token
                    info!(
                        "[{}] Done applying restriction, removing token... ({})",
                        token.token,
                        data.id.clone()
                    );
                    redis_conn
                        .hdel(KLIBRARIAN_INVITE_TOKEN, token.token.clone())
                        .await
                        .unwrap_or(0);

                    Ok(())
                }
                Err(_) => {
                    info!(
                        "[{}] Failed applying restriction... ({})",
                        token.token, data.id
                    );
                    anyhow::bail!("Failed to apply user restriction")
                }
            }
        }
        Err(error) => {
            anyhow::bail!("{}", error)
        }
    }
}

pub async fn apply_invite_token(
    State(state): State<AppState>,
    Path(token): Path<String>,
    Json(request): Json<InviteTokenApplicationRequest>,
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
    let mut redis_conn = state
        .redis
        .get_multiplexed_async_connection()
        .await
        .unwrap();

    info!("Applying invite token: {}", token);
    let data: Result<String, _> = redis_conn
        .hget(KLIBRARIAN_INVITE_TOKEN, token.clone())
        .await;

    match data {
        Ok(data) => {
            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", "application/json".parse().unwrap());

            let raw_val: InviteToken = serde_json::from_str(&data).unwrap();
            info!("[{}] Found token, checking if expired", token);

            match remove_token_or(&mut redis_conn, &raw_val).await {
                Ok(_) => {
                    info!("[{}] Found active, registering...", token);
                    let komga = KomgaClient::instance();

                    let res =
                        create_user_in_komga(&mut redis_conn, &komga, &raw_val, &request).await;

                    match res {
                        Ok(_) => {
                            // wrap the json in a {"ok": true, "data": {}} object
                            let mut komga_host = komga.get_host();

                            if let Ok(komga_hostname) = std::env::var("KOMGA_HOSTNAME")
                                && !komga_hostname.trim().is_empty()
                            {
                                komga_host = komga_hostname.trim().to_owned();
                            }

                            let wrapped_json: Value = serde_json::json!({
                                "ok": true,
                                "data": serde_json::json!({
                                    "host": komga_host,
                                })
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
                                "error": format!("Failed to create user: {}", error)
                            });

                            (
                                StatusCode::BAD_REQUEST,
                                headers,
                                serde_json::to_string(&wrapped_json).unwrap(),
                            )
                        }
                    }
                }
                Err(_) => {
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
            }
        }
        Err(_) => {
            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", "application/json".parse().unwrap());

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
    }
}

pub async fn get_all_invite_token(
    _: AuthToken,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut redis_conn = state
        .redis
        .get_multiplexed_async_connection()
        .await
        .unwrap();

    let all_keys: HashMap<String, String> = redis_conn
        .hgetall(KLIBRARIAN_INVITE_TOKEN)
        .await
        .unwrap_or(HashMap::new());

    let mut merged_token = vec![];
    for (_, value) in all_keys {
        let raw_val: InviteToken = serde_json::from_str(&value).unwrap();
        merged_token.push(raw_val);
    }

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());

    // wrap the json in a {"ok": true, "data": {}} object
    let wrapped_json: Value = serde_json::json!({
        "ok": true,
        "data": merged_token,
    });

    (
        StatusCode::OK,
        headers,
        serde_json::to_string(&wrapped_json).unwrap(),
    )
}

pub fn invite_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            axum::routing::get(get_all_invite_token).post(create_invite_token),
        )
        .route(
            "/:token",
            axum::routing::get(get_invite_token).delete(delete_invite_token),
        )
        .route("/:token/apply", axum::routing::post(apply_invite_token))
        .route("/config", axum::routing::get(get_invite_config))
        .with_state(state)
}
