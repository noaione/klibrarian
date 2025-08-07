const USER_AGENT: &str = "K-Librarian/0.2.0 (+https://github.com/noaione/klibrarian)";

#[derive(Debug)]
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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct KomgaMinimalLibrary {
    pub id: String,
    pub name: String,
    pub unavailable: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct KomgaCommonErrorViolation {
    pub field_name: String,
    pub message: String,
}

impl std::fmt::Display for KomgaCommonErrorViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field_name, self.message)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct KomgaCommonError {
    timestamp: String,
    status: u16,
    pub error: String,
    pub message: String,
    path: String,
}

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

    pub fn instance() -> Self {
        let komga_host = std::env::var("KOMGA_HOST").expect("KOMGA_HOST not set");
        let komga_username = std::env::var("KOMGA_USERNAME").expect("KOMGA_USERNAME not set");
        let komga_password = std::env::var("KOMGA_PASSWORD").expect("KOMGA_PASSWORD not set");

        Self::new(
            komga_host.clone(),
            komga_username.clone(),
            komga_password.clone(),
        )
    }

    pub async fn get_me(&self) -> anyhow::Result<KomgaUser> {
        let client = reqwest::Client::new();
        let res = client
            .get(format!("{}/api/v2/users/me", self.url))
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?;

        let user: KomgaUser = res.json().await?;

        Ok(user)
    }

    pub async fn create_user(&self, user: KomgaUserCreate) -> Result<KomgaUser, KomgaCommonError> {
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

            Err(error)
        }
    }

    pub async fn apply_user_restriction(
        &self,
        user_id: String,
        option: KomgaUserCreateOption,
    ) -> anyhow::Result<()> {
        let res = self
            .client
            .patch(format!("{}/api/v2/users/{}", self.url, user_id))
            .basic_auth(&self.username, Some(&self.password))
            .json(&option)
            .send()
            .await?;

        let status_code = res.status();

        if status_code.is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to apply user restriction"))
        }
    }

    pub async fn get_sharing_labels(&self) -> anyhow::Result<Vec<String>> {
        let res = self
            .client
            .get(format!("{}/api/v1/sharing-labels", self.url))
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?;

        let labels: Vec<String> = res.json().await?;

        Ok(labels)
    }

    pub async fn get_libraries(&self) -> anyhow::Result<Vec<KomgaMinimalLibrary>> {
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
