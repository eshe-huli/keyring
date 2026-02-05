//! Configuration — TOML-based node configuration

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// Node name
    pub name: String,

    /// Data directory
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,

    /// Transport configurations
    #[serde(default)]
    pub transports: Vec<TransportConfig>,

    /// Bootstrap peers
    #[serde(default)]
    pub bootstrap_peers: Vec<String>,

    /// API configuration
    #[serde(default)]
    pub api: ApiConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransportConfig {
    #[serde(rename = "type")]
    pub transport_type: String,
    #[serde(default)]
    pub listen: Option<String>,
    #[serde(default)]
    pub options: std::collections::HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiConfig {
    #[serde(default = "default_grpc_addr")]
    pub grpc_addr: String,
    pub http_addr: Option<String>,
    pub unix_socket: Option<String>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            grpc_addr: default_grpc_addr(),
            http_addr: None,
            unix_socket: Some("/tmp/keyring.sock".to_string()),
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Generate default config for a new node
    pub fn default_for(name: &str) -> Self {
        Self {
            name: name.to_string(),
            data_dir: default_data_dir(),
            transports: vec![
                TransportConfig {
                    transport_type: "quic".to_string(),
                    listen: Some("0.0.0.0:4433".to_string()),
                    options: std::collections::HashMap::new(),
                },
            ],
            bootstrap_peers: vec![],
            api: ApiConfig::default(),
        }
    }
}

fn default_data_dir() -> PathBuf {
    directories::ProjectDirs::from("", "", "keyring")
        .map(|d| d.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("~/.keyring"))
}

fn default_grpc_addr() -> String {
    "[::1]:4434".to_string()
}
