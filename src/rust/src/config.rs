use std::{env, path::PathBuf};

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
}

