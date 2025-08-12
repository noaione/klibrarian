use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Main configuration structure for k-librarian
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The host address to bind the web server to
    pub host: String,
    /// The port to bind the web server to
    pub port: u16,
    /// Authentication token for accessing the admin panel
    pub token: String,
    /// Path to the database file (relative or absolute)
    #[serde(rename = "db-path")]
    pub db_path: PathBuf,
    /// Komga instance configuration (required)
    pub komga: KomgaConfig,
    /// Navidrome instance configuration (optional)
    pub navidrome: Option<NavidromeConfig>,
}

/// Komga instance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KomgaConfig {
    /// Host URL of the Komga instance
    pub host: String,
    /// Username for Komga authentication
    pub username: String,
    /// Password for Komga authentication
    pub password: String,
    /// Optional actual hostname if running behind a reverse proxy
    pub hostname: Option<String>,
}

/// Navidrome instance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavidromeConfig {
    /// Host URL of the Navidrome instance
    pub host: String,
    /// Username for Navidrome authentication
    pub username: String,
    /// Password for Navidrome authentication
    pub password: String,
    /// Optional actual hostname if running behind a reverse proxy
    pub hostname: Option<String>,
}

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;

        Self::from_str(&content)
    }

    /// Parse configuration from a TOML string
    pub fn from_str(content: &str) -> Result<Self> {
        toml::from_str(content).with_context(|| "Failed to parse TOML configuration")
    }

    /// Get the effective Komga hostname (hostname field or host field)
    pub fn komga_hostname(&self) -> &str {
        self.komga.hostname.as_deref().unwrap_or(&self.komga.host)
    }

    /// Get the effective Navidrome hostname (hostname field or host field)
    pub fn navidrome_hostname(&self) -> Option<&str> {
        self.navidrome
            .as_ref()
            .map(|n| n.hostname.as_deref().unwrap_or(&n.host))
    }

    /// Check if Navidrome is configured
    pub fn has_navidrome(&self) -> bool {
        self.navidrome.is_some()
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate host is not empty
        if self.host.trim().is_empty() {
            anyhow::bail!("Host cannot be empty");
        }

        // Validate port is reasonable
        if self.port == 0 {
            anyhow::bail!("Port cannot be 0");
        }

        // Validate token is not empty
        if self.token.trim().is_empty() {
            anyhow::bail!("Auth token cannot be empty");
        }

        // Validate Komga configuration
        if self.komga.host.trim().is_empty() {
            anyhow::bail!("Komga host cannot be empty");
        }
        if self.komga.username.trim().is_empty() {
            anyhow::bail!("Komga username cannot be empty");
        }
        if self.komga.password.trim().is_empty() {
            anyhow::bail!("Komga password cannot be empty");
        }

        // Validate Navidrome configuration if present
        if let Some(ref navidrome) = self.navidrome {
            if navidrome.host.trim().is_empty() {
                anyhow::bail!("Navidrome host cannot be empty");
            }
            if navidrome.username.trim().is_empty() {
                anyhow::bail!("Navidrome username cannot be empty");
            }
            if navidrome.password.trim().is_empty() {
                anyhow::bail!("Navidrome password cannot be empty");
            }
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 5148,
            token: "this-is-your-auth-token".to_string(),
            db_path: PathBuf::from("./.klibrarian/database.sqlite"),
            komga: KomgaConfig {
                host: "https://demo.komga.org".to_string(),
                username: "demo@komga.org".to_string(),
                password: "demo".to_string(),
                hostname: None,
            },
            navidrome: None,
        }
    }
}
