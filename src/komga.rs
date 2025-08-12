use crate::{config::KomgaConfig, database::KomgaInviteOption};

const USER_AGENT: &str = concat!(
    "K-Librarian/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/noaione/klibrarian)"
);

pub struct KomgaClient {
    url: String,
    username: String,
    password: String,
    client: reqwest::Client,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct KomgaUser {
    pub id: String,
    pub email: String,
    pub roles: Vec<String>,
    #[serde(rename = "sharedAllLibraries")]
    pub shared_all_libraries: bool,
    #[serde(rename = "sharedLibrariesIds")]
    pub shared_libraries_ids: Vec<String>,
    #[serde(rename = "labelsAllow")]
    pub labels_allow: Vec<String>,
    #[serde(rename = "labelsExclude")]
    pub labels_exclude: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct KomgaUserCreate {
    pub email: String,
    pub password: String,
    pub roles: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct KomgaUserCreateOptionSharedLibraries {
    pub all: bool,
    #[serde(rename = "libraryIds")]
    pub library_ids: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct KomgaUserCreateOption {
    #[serde(rename = "labelsAllow")]
    pub labels_allow: Option<Vec<String>>,
    #[serde(rename = "labelsExclude")]
    pub labels_exclude: Option<Vec<String>>,
    #[serde(rename = "sharedLibraries")]
    pub shared_libraries: Option<KomgaUserCreateOptionSharedLibraries>,
}

impl From<KomgaInviteOption> for KomgaUserCreateOption {
    fn from(val: KomgaInviteOption) -> Self {
        KomgaUserCreateOption {
            labels_allow: val.labels_allow,
            labels_exclude: val.labels_exclude,
            shared_libraries: val.shared_libraries,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct KomgaMinimalLibrary {
    pub id: String,
    pub name: String,
    pub unavailable: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct KomgaCommonErrorViolation {
    pub field_name: String,
    pub message: String,
}

impl std::fmt::Display for KomgaCommonErrorViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field_name, self.message)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct KomgaViolationsError {
    pub violations: Vec<KomgaCommonErrorViolation>,
}

impl std::fmt::Display for KomgaViolationsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut violations = String::new();

        for violation in &self.violations {
            violations.push_str(&format!("{violation}\n"));
        }

        write!(f, "{violations}")
    }
}

impl std::error::Error for KomgaViolationsError {}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct KomgaCommonError {
    timestamp: String,
    status: u16,
    pub error: String,
    pub message: String,
    path: String,
}

impl std::error::Error for KomgaCommonError {}

impl std::fmt::Display for KomgaCommonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.error, self.message)
    }
}

impl KomgaClient {
    pub fn new(url: String, username: String, password: String) -> Self {
        let client = reqwest::ClientBuilder::new()
            .user_agent(USER_AGENT)
            .build()
            .unwrap();

        Self {
            url,
            username,
            password,
            client,
        }
    }

    pub fn instance(config: &KomgaConfig) -> Self {
        Self::new(
            config.host.clone(),
            config.username.clone(),
            config.password.clone(),
        )
    }

    pub async fn get_me(&self) -> Result<KomgaUser, KomgaError> {
        let client = reqwest::Client::new();
        let res = client
            .get(format!("{}/api/v2/users/me", self.url))
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?;

        let user: KomgaUser = res.json().await?;

        Ok(user)
    }

    pub async fn create_user(&self, user: KomgaUserCreate) -> Result<KomgaUser, KomgaError> {
        let res = self
            .client
            .post(format!("{}/api/v2/users", self.url))
            .basic_auth(&self.username, Some(&self.password))
            .json(&user)
            .send()
            .await
            .unwrap();

        if res.status().is_success() {
            let user: KomgaUser = res.json().await.unwrap();

            Ok(user)
        } else {
            let error: KomgaCommonError = res.json().await.unwrap();

            Err(KomgaError::Common(error))
        }
    }

    pub async fn apply_user_restriction(
        &self,
        user_id: &str,
        option: &KomgaUserCreateOption,
    ) -> Result<(), KomgaError> {
        let res = self
            .client
            .patch(format!("{}/api/v2/users/{}", self.url, user_id))
            .basic_auth(&self.username, Some(&self.password))
            .json(option)
            .send()
            .await?;

        let status_code = res.status();

        if status_code.is_success() {
            Ok(())
        } else {
            Err(KomgaError::ApplyUserRestriction)
        }
    }

    pub async fn get_sharing_labels(&self) -> Result<Vec<String>, KomgaError> {
        let res = self
            .client
            .get(format!("{}/api/v1/sharing-labels", self.url))
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?;

        let labels: Vec<String> = res.json().await?;

        Ok(labels)
    }

    pub async fn get_libraries(&self) -> Result<Vec<KomgaMinimalLibrary>, KomgaError> {
        let res = self
            .client
            .get(format!("{}/api/v1/libraries", self.url))
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?;

        let libraries: Vec<KomgaMinimalLibrary> = res.json().await?;

        Ok(libraries)
    }

    pub fn get_host(&self) -> String {
        self.url.clone()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum KomgaError {
    #[error("failed to connect to Komga: {0}")]
    Connection(#[from] reqwest::Error),
    #[error("failed to parse Komga response: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("Komga returned an error: {0}")]
    Common(#[from] KomgaCommonError),
    #[error("Komga returned a violation error: {0}")]
    Violation(#[from] KomgaViolationsError),
    #[error("failed to apply user restriction")]
    ApplyUserRestriction,
    #[error("unknown error occurred")]
    Unknown,
}
