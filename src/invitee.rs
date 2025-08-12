use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    AppState,
    database::InviteToken,
    komga::{self, KomgaUserCreate},
    navidrome,
};

const KOMGA_DEFAULT_ROLES: &[&str] = &["USER", "FILE_DOWNLOAD", "PAGE_STREAMING"];

#[derive(serde::Serialize, serde::Deserialize, garde::Validate)]
pub struct InviteTokenApplicationPayload {
    #[garde(email)]
    email: String,
    #[garde(length(min = 6))]
    password: String,
    #[garde(custom(validate_username()))]
    username: String, // although, ignored in Komga
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum UserCreationError {
    #[error("wrong invite kind for user creation: {0}, expected {1}")]
    WrongInviteKind(String, &'static str),
    #[error("failed to create user in Komga: {0}")]
    KomgaError(#[from] komga::KomgaError),
    #[error("failed to create user in Navidrome: {0}")]
    NavidromeError(#[from] navidrome::NavidromeError),
    #[error("failed to communicate with the database: {0}")]
    DatabaseError(#[from] crate::database::LocalDatabaseError),
    #[error("client {0} is unavailable for user creation")]
    ClientUnavailable(&'static str),
    #[error("unknown error during user creation")]
    #[expect(dead_code)]
    UnknownError,
}

async fn create_user_in_komga(
    database: &Arc<crate::database::LocalDatabase>,
    komga: &Arc<komga::KomgaClient>,
    token: &InviteToken,
    payload: &InviteTokenApplicationPayload,
) -> Result<(), UserCreationError> {
    let (user_create, user_id, create_option) = match token {
        InviteToken::Navidrome { .. } => {
            return Err(UserCreationError::WrongInviteKind(
                "Navidrome".to_string(),
                "Komga",
            ));
        }
        InviteToken::Komga { option, uuid, .. } => {
            let roles = option.roles.clone().unwrap_or(
                KOMGA_DEFAULT_ROLES
                    .to_vec()
                    .iter()
                    .map(|x| x.to_string())
                    .collect(),
            );

            let user_create = KomgaUserCreate {
                email: payload.email.clone(),
                password: payload.password.clone(),
                roles,
            };

            (user_create, uuid.clone(), option.clone())
        }
    };

    match user_id {
        Some(uuid) => {
            tracing::info!(
                "[{} / {}] User already created, applying restrictions",
                token.token(),
                uuid
            );

            komga
                .apply_user_restriction(&uuid, &create_option.into())
                .await?;

            // Delete the invite token after successful user creation
            tracing::info!(
                "[{}] Deleting invite token after user restriction application",
                token.token()
            );
            database.delete_invite(token.token()).await?;
            Ok(())
        }
        None => {
            tracing::info!(
                "[{}] Creating new user with email: {}",
                token.token(),
                &payload.email
            );

            let user = komga.create_user(user_create).await?;
            // Update database with the new user ID
            tracing::info!(
                "[{}] Applying restrictions for: {}",
                token.token(),
                &user.id
            );
            database.apply_user_id(token.token(), &user.id).await?;

            // Apply user restrictions
            komga
                .apply_user_restriction(&user.id, &create_option.into())
                .await?;

            // Delete the invite token after successful user creation
            tracing::info!(
                "[{}] Deleting invite token after user creation",
                token.token()
            );
            database.delete_invite(token.token()).await?;

            tracing::info!(
                "[{}] User created successfully with ID: {}",
                token.token(),
                user.id
            );
            Ok(())
        }
    }
}

async fn create_user_in_navidrome(
    database: &Arc<crate::database::LocalDatabase>,
    navidrome: &Arc<Mutex<navidrome::NavidromeClient>>,
    token: &InviteToken,
    payload: &InviteTokenApplicationPayload,
) -> Result<(), UserCreationError> {
    let (user_create, user_id, create_option) = match token {
        InviteToken::Komga { .. } => {
            return Err(UserCreationError::WrongInviteKind(
                "Komga".to_string(),
                "Navidrome",
            ));
        }
        InviteToken::Navidrome { option, uuid, .. } => {
            let user_create = navidrome::NavidromeUserCreate::new(
                &payload.username,
                &payload.email,
                &payload.password,
                option.is_admin,
            );

            (user_create, uuid.clone(), option.clone())
        }
    };

    let mut navidrome_client = navidrome.lock().await;

    match user_id {
        Some(uuid) => {
            tracing::info!(
                "[{} / {}] User already created, applying restrictions",
                token.token(),
                uuid
            );

            navidrome_client
                .apply_user_library(&uuid, &create_option.into())
                .await?;

            // Delete the invite token after successful user creation
            tracing::info!(
                "[{}] Deleting invite token after user restriction application",
                token.token()
            );
            database.delete_invite(token.token()).await?;
            Ok(())
        }
        None => {
            tracing::info!(
                "[{}] Creating new user with email: {}",
                token.token(),
                &payload.email
            );

            let user = navidrome_client.create_user(user_create).await?;
            // Update database with the new user ID
            tracing::info!(
                "[{}] Applying restrictions for: {}",
                token.token(),
                &user.id
            );
            database.apply_user_id(token.token(), &user.id).await?;

            // Apply user restrictions
            navidrome_client
                .apply_user_library(&user.id, &create_option.into())
                .await?;

            // Delete the invite token after successful user creation
            tracing::info!(
                "[{}] Deleting invite token after user creation",
                token.token()
            );
            database.delete_invite(token.token()).await?;

            tracing::info!(
                "[{}] User created successfully with ID: {}",
                token.token(),
                user.id
            );
            Ok(())
        }
    }
}

pub async fn create_user_in(
    state: &AppState,
    token: &InviteToken,
    payload: &InviteTokenApplicationPayload,
) -> Result<String, UserCreationError> {
    match token {
        InviteToken::Komga { .. } => {
            create_user_in_komga(&state.db, &state.komga, token, payload).await?;

            // get the host
            let host = state.config.komga_hostname();

            Ok(host.to_string())
        }
        InviteToken::Navidrome { .. } => {
            match (&state.navidrome, state.config.navidrome_hostname()) {
                (Some(navidrome), Some(navidrome_host)) => {
                    create_user_in_navidrome(&state.db, &navidrome, token, payload).await?;

                    Ok(navidrome_host.to_string())
                }
                _ => Err(UserCreationError::ClientUnavailable("Navidrome")),
            }
        }
    }
}

fn validate_username() -> impl FnOnce(&str, &()) -> garde::Result + 'static {
    move |value: &str, _| {
        let trimmed_value = value.trim();
        if trimmed_value.is_empty() {
            Err(garde::Error::new("Username cannot be empty"))
        } else if trimmed_value.len() < 1 {
            Err(garde::Error::new(
                "Username must be at least 1 character long",
            ))
        }
        // only check alphanumeric, dash, and underscore
        else if !trimmed_value
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            Err(garde::Error::new(
                "Username can only contain alphanumeric characters, dashes, and underscores",
            ))
        } else {
            Ok(())
        }
    }
}
