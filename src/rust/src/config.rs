use std::{env, net::IpAddr, path::PathBuf};

use thiserror::Error;

const DEVELOPMENT_SHARED_SECRET: &str = "development-shared-secret";

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("shared secret must not be empty")]
    EmptySharedSecret,
    #[error("shared secret must be replaced before binding to non-loopback interfaces")]
    DevelopmentSecretOnExternalInterface,
    #[error("public endpoint must start with http:// or https://")]
    InvalidPublicEndpoint,
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub node_id: String,
    pub node_role: String,
    pub public_endpoint: String,
    pub db_path: PathBuf,
    pub shared_secret: String,
    pub bootstrap_peers: Vec<String>,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let host = env::var("STRATUS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("STRATUS_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(8080);
        let node_id = env::var("STRATUS_NODE_ID").unwrap_or_else(|_| "node-alpha".to_string());
        let node_role = env::var("STRATUS_NODE_ROLE").unwrap_or_else(|_| "primary".to_string());
        let db_path = env::var("STRATUS_DB_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("../../data/sql/node-alpha.db"));
        let shared_secret = env::var("STRATUS_SHARED_SECRET")
            .unwrap_or_else(|_| "development-shared-secret".to_string());
        let bootstrap_peers = env::var("STRATUS_BOOTSTRAP_PEERS")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        let public_endpoint = env::var("STRATUS_PUBLIC_ENDPOINT")
            .unwrap_or_else(|_| format!("http://127.0.0.1:{port}"));

        Self {
            host,
            port,
            node_id,
            node_role,
            public_endpoint,
            db_path,
            shared_secret,
            bootstrap_peers,
        }
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn uses_development_secret(&self) -> bool {
        self.shared_secret == DEVELOPMENT_SHARED_SECRET
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.shared_secret.trim().is_empty() {
            return Err(ConfigError::EmptySharedSecret);
        }

        if !(self.public_endpoint.starts_with("http://") || self.public_endpoint.starts_with("https://")) {
            return Err(ConfigError::InvalidPublicEndpoint);
        }

        if self.uses_development_secret() && !is_loopback_host(&self.host) {
            return Err(ConfigError::DevelopmentSecretOnExternalInterface);
        }

        Ok(())
    }
}

fn is_loopback_host(host: &str) -> bool {
    if host == "localhost" {
        return true;
    }

    host.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback())
}

#[cfg(test)]
mod tests {
    use super::{AppConfig, ConfigError};
    use std::path::PathBuf;

    fn config_with(host: &str, shared_secret: &str, public_endpoint: &str) -> AppConfig {
        AppConfig {
            host: host.to_string(),
            port: 8080,
            node_id: "node-test".to_string(),
            node_role: "primary".to_string(),
            public_endpoint: public_endpoint.to_string(),
            db_path: PathBuf::from("/tmp/node-test.db"),
            shared_secret: shared_secret.to_string(),
            bootstrap_peers: vec![],
        }
    }

    #[test]
    fn validate_accepts_loopback_with_development_secret() {
        let config = config_with("127.0.0.1", "development-shared-secret", "http://127.0.0.1:8080");
        assert!(config.validate().is_ok());
        assert!(config.uses_development_secret());
    }

    #[test]
    fn validate_rejects_development_secret_on_external_interface() {
        let config = config_with("0.0.0.0", "development-shared-secret", "http://service:8080");
        assert_eq!(config.validate(), Err(ConfigError::DevelopmentSecretOnExternalInterface));
    }

    #[test]
    fn validate_rejects_invalid_public_endpoint() {
        let config = config_with("127.0.0.1", "real-secret", "service:8080");
        assert_eq!(config.validate(), Err(ConfigError::InvalidPublicEndpoint));
    }

    #[test]
    fn validate_rejects_empty_secret() {
        let config = config_with("127.0.0.1", "   ", "http://127.0.0.1:8080");
        assert_eq!(config.validate(), Err(ConfigError::EmptySharedSecret));
    }
}

