use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};

use crate::{config::NavidromeConfig, database::NavidromeInviteOption};

const USER_AGENT: &str = "K-Librarian/0.3.0 (+https://github.com/noaione/klibrarian)";

#[derive(Debug, serde::Deserialize)]
struct NavidromeLoginResponse {
    token: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct NavidromeMinimalLibrary {
    id: u64,
    name: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct NavidromeUser {
    pub id: String,
}

#[derive(Debug, serde::Serialize)]
pub struct NavidromeUserCreate {
    email: String,
    password: String,
    #[serde(rename = "isAdmin")]
    is_admin: bool,
    name: String,
    #[serde(rename = "userName")]
    username: String,
}

impl NavidromeUserCreate {
    pub fn new(
        username: impl Into<String>,
        email: impl Into<String>,
        password: impl Into<String>,
        is_admin: bool,
    ) -> Self {
        let username = username.into();
        let email = email.into();
        let password = password.into();
        let name = username.clone();
        Self {
            email,
            password,
            is_admin,
            name,
            username,
        }
    }
}

#[derive(serde::Serialize, Debug)]
pub struct NavidromeUserCreateOption {
    #[serde(rename = "libraryIds")]
    library_ids: Vec<u64>,
}

impl From<NavidromeInviteOption> for NavidromeUserCreateOption {
    fn from(option: NavidromeInviteOption) -> Self {
        NavidromeUserCreateOption {
            library_ids: option.library_ids,
        }
    }
}

pub struct NavidromeClient {
    client: reqwest::Client,
    config: NavidromeConfig,
    token: String,
    claims: MinimalJwtClaims,
}

impl NavidromeClient {
    pub async fn new(config: &NavidromeConfig) -> Result<Self, NavidromeError> {
        let client = reqwest::Client::builder().user_agent(USER_AGENT).build()?;

        // login to Navidrome to get the JWT token
        let login_url = format!("{}/auth/login", config.host);

        let login_response = client
            .post(&login_url)
            .json(&serde_json::json!({
                "username": config.username,
                "password": config.password,
            }))
            .send()
            .await?;

        let login_json: NavidromeLoginResponse = login_response.json().await?;

        // decode jwt
        let decoded_claims = decode_jwt(&login_json.token)?;

        Ok(Self {
            client,
            config: config.clone(),
            token: login_json.token,
            claims: decoded_claims,
        })
    }

    pub fn claims(&self) -> &MinimalJwtClaims {
        &self.claims
    }

    fn token(&self) -> String {
        format!("Bearer {}", self.token)
    }

    pub async fn get_library(&mut self) -> Result<Vec<NavidromeMinimalLibrary>, NavidromeError> {
        let url = format!("{}/api/library", self.config.host);
        let response = self
            .client
            .get(&url)
            .query(&[
                ("_end", "-1"),
                ("_start", "0"),
                ("_sort", "id"),
                ("_order", "asc"),
            ])
            .header("x-nd-authorization", self.token())
            .send()
            .await?;

        // get x-nd-authorization header
        if let Some(auth_header) = response.headers().get("x-nd-authorization")
            && let Ok(auth_value) = auth_header.to_str()
        {
            self.token = auth_value.to_string();
        }

        let libraries: Vec<NavidromeMinimalLibrary> = response.json().await?;

        Ok(libraries)
    }

    pub async fn create_user(
        &mut self,
        user: NavidromeUserCreate,
    ) -> Result<NavidromeUser, NavidromeError> {
        let url = format!("{}/api/user", self.config.host);
        let response = self
            .client
            .post(&url)
            .json(&user)
            .header("x-nd-authorization", self.token())
            .send()
            .await?;

        // get x-nd-authorization header
        if let Some(auth_header) = response.headers().get("x-nd-authorization")
            && let Ok(auth_value) = auth_header.to_str()
        {
            self.token = auth_value.to_string();
        }

        let resp = response.json::<NavidromeUser>().await?;
        Ok(resp)
    }

    pub async fn apply_user_library(
        &mut self,
        user_id: &str,
        option: &NavidromeUserCreateOption,
    ) -> Result<(), NavidromeError> {
        let url = format!("{}/api/user/{}/library", self.config.host, user_id);
        let response = self
            .client
            .put(&url)
            .json(option)
            .header("x-nd-authorization", self.token())
            .send()
            .await?;

        // get x-nd-authorization header
        if let Some(auth_header) = response.headers().get("x-nd-authorization")
            && let Ok(auth_value) = auth_header.to_str()
        {
            self.token = auth_value.to_string();
        }

        if response.status().is_success() {
            Ok(())
        } else {
            Err(NavidromeError::ApplyUserRestrictionError)
        }
    }
}

#[derive(serde::Deserialize)]
#[expect(dead_code)]
pub struct MinimalJwtClaims {
    pub adm: bool,
    exp: i64,
    iat: i64,
    iss: String,
    sub: String,
    uid: String,
}

fn decode_jwt(token: &str) -> Result<MinimalJwtClaims, NavidromeError> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(NavidromeError::JWTDecodeError("Invalid JWT format"));
    }

    let payload = parts[1];
    let decoded_payload = BASE64_URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|_| NavidromeError::JWTDecodeError("Failed to decode JWT payload"))?;

    let claims: MinimalJwtClaims = serde_json::from_slice(&decoded_payload)
        .map_err(|_| NavidromeError::JWTDecodeError("Failed to parse JWT payload"))?;

    Ok(claims)
}

#[derive(Debug, thiserror::Error)]
pub enum NavidromeError {
    #[error("failed to connect to Navidrome: {0}")]
    ConnectionError(#[from] reqwest::Error),
    #[error("failed to parse Navidrome response: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("failed to decode JWT token: {0}")]
    JWTDecodeError(&'static str),
    #[error("failed to apply user restriction")]
    ApplyUserRestrictionError,
    #[error("unknown error occurred")]
    UnknownError,
}
