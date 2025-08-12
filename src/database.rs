use std::path::PathBuf;

use sqlx::sqlite::SqliteConnectOptions;

use crate::komga::KomgaUserCreateOptionSharedLibraries;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct KomgaInviteOption {
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

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct NavidromeInviteOption {
    #[serde(rename = "isAdmin")]
    pub is_admin: bool,
    #[serde(rename = "expiresAt")]
    pub expire_at: Option<u64>,
    #[serde(rename = "libraryIds")]
    pub library_ids: Vec<u64>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind")]
pub enum InviteToken {
    #[serde(rename = "komga")]
    Komga {
        token: uuid::Uuid,
        option: KomgaInviteOption,
        uuid: Option<String>,
    },
    #[serde(rename = "navidrome")]
    Navidrome {
        token: uuid::Uuid,
        option: NavidromeInviteOption,
        uuid: Option<String>,
    },
}

impl InviteToken {
    pub fn token(&self) -> uuid::Uuid {
        match self {
            InviteToken::Komga { token, .. } => *token,
            InviteToken::Navidrome { token, .. } => *token,
        }
    }

    pub fn is_expired(&self) -> bool {
        let unix_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        match self {
            InviteToken::Komga { option, .. } => {
                if let Some(expire_at) = option.expire_at {
                    unix_time > expire_at
                } else {
                    false
                }
            }
            InviteToken::Navidrome { option, .. } => {
                if let Some(expire_at) = option.expire_at {
                    unix_time > expire_at
                } else {
                    false
                }
            }
        }
    }

    pub fn option(&self) -> serde_json::Value {
        match self {
            InviteToken::Komga { option, .. } => serde_json::to_value(option).unwrap(),
            InviteToken::Navidrome { option, .. } => serde_json::to_value(option).unwrap(),
        }
    }

    pub fn option_str(&self) -> Result<String, serde_json::Error> {
        match self {
            InviteToken::Komga { option, .. } => serde_json::to_string(option),
            InviteToken::Navidrome { option, .. } => serde_json::to_string(option),
        }
    }

    pub fn uuid(&self) -> Option<&str> {
        match self {
            InviteToken::Komga { uuid, .. } => uuid.as_deref(),
            InviteToken::Navidrome { uuid, .. } => uuid.as_deref(),
        }
    }

    pub fn kind(&self) -> &str {
        match self {
            InviteToken::Komga { .. } => "komga",
            InviteToken::Navidrome { .. } => "navidrome",
        }
    }

    pub fn create_komga(option: KomgaInviteOption) -> Self {
        InviteToken::Komga {
            token: uuid::Uuid::new_v4(),
            option,
            uuid: None,
        }
    }

    pub fn create_navidrome(option: NavidromeInviteOption) -> Self {
        InviteToken::Navidrome {
            token: uuid::Uuid::new_v4(),
            option,
            uuid: None,
        }
    }
}

#[derive(Debug)]
pub struct LocalDatabase {
    pool: sqlx::Pool<sqlx::Sqlite>,
}

impl LocalDatabase {
    pub async fn new(path: &PathBuf) -> Result<Self, sqlx::Error> {
        tracing::info!("📂 Using database at: {}", path.display());
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true);

        let pool = sqlx::SqlitePool::connect_with(options).await?;
        Ok(Self { pool })
    }

    pub async fn setup(&self) -> Result<(), sqlx::Error> {
        // Create tables if they do not exist
        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS invites (
                token TEXT PRIMARY KEY,
                option TEXT NOT NULL,
                uuid TEXT,
                kind TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )"#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn add_invite(&self, invite: &InviteToken) -> Result<(), LocalDatabaseError> {
        let option_json = invite.option_str()?;

        // execute insert query
        sqlx::query(
            r#"
            INSERT INTO invites (token, option, uuid, kind)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(invite.token().to_string())
        .bind(option_json)
        .bind(invite.uuid().map(|s| s.to_string()))
        .bind(invite.kind())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_invite(
        &self,
        token: uuid::Uuid,
    ) -> Result<Option<InviteToken>, LocalDatabaseError> {
        let row: Option<(String, String, Option<String>, String)> = sqlx::query_as(
            r#"
            SELECT token, option, uuid, kind FROM invites
            WHERE token = ?
            "#,
        )
        .bind(token.to_string())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let invite = cast_sql_row_to_invite_token(row)?;
                Ok(Some(invite))
            }
            None => Ok(None),
        }
    }

    pub async fn delete_invite(&self, token: uuid::Uuid) -> Result<(), LocalDatabaseError> {
        sqlx::query("DELETE FROM invites WHERE token = ?")
            .bind(token.to_string())
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn apply_user_id(
        &self,
        token: uuid::Uuid,
        user_id: &str,
    ) -> Result<(), LocalDatabaseError> {
        sqlx::query("UPDATE invites SET uuid = ? WHERE token = ?")
            .bind(user_id)
            .bind(token.to_string())
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_all_invites(&self) -> Result<Vec<InviteToken>, LocalDatabaseError> {
        let rows: Vec<(String, String, Option<String>, String)> = sqlx::query_as(
            r#"
            SELECT token, option, uuid, kind FROM invites
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut invites: Vec<InviteToken> = Vec::new();
        for row in rows {
            // do this to bubble up any errors
            let invite = cast_sql_row_to_invite_token(row)?;
            invites.push(invite);
        }

        Ok(invites)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum LocalDatabaseError {
    #[error("database error")]
    SqlError(#[from] sqlx::Error),
    #[error("JSON processing error")]
    JsonError(#[from] serde_json::Error),
    #[error("invalid invite token, expected UUID format")]
    InvalidInviteToken,
    #[error("unknown token kind: {0}")]
    UnknownTokenKind(String),
}

fn cast_sql_row_to_invite_token(
    row: (String, String, Option<String>, String),
) -> Result<InviteToken, LocalDatabaseError> {
    let (token, option_str, uuid, kind) = row;
    let token_uuid =
        uuid::Uuid::parse_str(&token).map_err(|_| LocalDatabaseError::InvalidInviteToken)?;

    match kind.to_lowercase().as_str() {
        "komga" => {
            let option = serde_json::from_str::<KomgaInviteOption>(&option_str)?;
            Ok(InviteToken::Komga {
                token: token_uuid,
                option,
                uuid,
            })
        }
        "navidrome" => {
            let option = serde_json::from_str::<NavidromeInviteOption>(&option_str)?;
            Ok(InviteToken::Navidrome {
                token: token_uuid,
                option,
                uuid,
            })
        }
        _ => Err(LocalDatabaseError::UnknownTokenKind(kind)),
    }
}
