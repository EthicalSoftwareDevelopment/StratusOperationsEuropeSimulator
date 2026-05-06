use crate::config::AppConfig;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{fs, path::Path, path::PathBuf, time::Duration};
use thiserror::Error;
const INIT_SQL: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/sql/001_init.sql"));
#[derive(Debug, Error)]
pub enum DbError {
    #[error("database error: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("filesystem error: {0}")]
    Io(#[from] std::io::Error),
}
#[derive(Clone, Debug)]
pub struct Database {
    db_path: PathBuf,
}
#[derive(Debug, Serialize)]
pub struct NodeRecord {
    pub node_id: String,
    pub role: String,
    pub status: String,
    pub endpoint: String,
    pub metadata_json: String,
    pub last_seen_utc: String,
}
#[derive(Debug, Serialize)]
pub struct ReplicationStateRecord {
    pub peer_node_id: String,
    pub last_replicated_event_id: i64,
    pub last_sync_utc: Option<String>,
    pub status: String,
}
#[derive(Debug, Serialize)]
pub struct StatusSnapshot {
    pub node_count: i64,
    pub mission_record_count: i64,
    pub event_count: i64,
    pub replication_peer_count: i64,
    pub last_event_utc: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct LoggedEvent {
    pub event_id: i64,
    pub event_hash: String,
    pub signature: String,
    pub created_at_utc: String,
}
impl Database {
    pub fn new(db_path: impl Into<PathBuf>) -> Self {
        Self {
            db_path: db_path.into(),
        }
    }
    pub fn path(&self) -> &Path {
        &self.db_path
    }
    pub fn bootstrap(&self, config: &AppConfig) -> Result<(), DbError> {
        self.initialize()?;
        self.upsert_node(
            &config.node_id,
            &config.node_role,
            "online",
            &config.public_endpoint,
            r#"{"trust_zone":"simulation-core"}"#,
        )?;
        for peer in &config.bootstrap_peers {
            self.upsert_replication_state(peer)?;
        }
        Ok(())
    }
    pub fn initialize(&self) -> Result<(), DbError> {
        if let Some(parent) = self.db_path.parent() {
            fs::create_dir_all(parent)?;
        }
        self.with_connection(|connection| {
            connection.execute_batch(INIT_SQL)?;
            Ok(())
        })
    }
    pub fn upsert_node(
        &self,
        node_id: &str,
        role: &str,
        status: &str,
        endpoint: &str,
        metadata_json: &str,
    ) -> Result<(), DbError> {
        let now = utc_now();
        self.with_connection(|connection| {
            connection.execute(
                "INSERT INTO nodes (node_id, role, status, endpoint, metadata_json, last_seen_utc)\n                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)\n                 ON CONFLICT(node_id) DO UPDATE SET\n                     role = excluded.role,\n                     status = excluded.status,\n                     endpoint = excluded.endpoint,\n                     metadata_json = excluded.metadata_json,\n                     last_seen_utc = excluded.last_seen_utc",
                params![node_id, role, status, endpoint, metadata_json, now],
            )?;
            Ok(())
        })
    }
    pub fn upsert_replication_state(&self, peer_node_id: &str) -> Result<(), DbError> {
        self.with_connection(|connection| {
            connection.execute(
                "INSERT INTO replication_state (peer_node_id, status) VALUES (?1, 'planned')\n                 ON CONFLICT(peer_node_id) DO NOTHING",
                params![peer_node_id],
            )?;
            Ok(())
        })
    }
    pub fn list_nodes(&self) -> Result<Vec<NodeRecord>, DbError> {
        self.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT node_id, role, status, endpoint, metadata_json, last_seen_utc\n                 FROM nodes\n                 ORDER BY node_id ASC",
            )?;
            let rows = statement.query_map([], |row| {
                Ok(NodeRecord {
                    node_id: row.get(0)?,
                    role: row.get(1)?,
                    status: row.get(2)?,
                    endpoint: row.get(3)?,
                    metadata_json: row.get(4)?,
                    last_seen_utc: row.get(5)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
        })
    }
    pub fn list_replication_state(&self) -> Result<Vec<ReplicationStateRecord>, DbError> {
        self.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT peer_node_id, last_replicated_event_id, last_sync_utc, status\n                 FROM replication_state\n                 ORDER BY peer_node_id ASC",
            )?;
            let rows = statement.query_map([], |row| {
                Ok(ReplicationStateRecord {
                    peer_node_id: row.get(0)?,
                    last_replicated_event_id: row.get(1)?,
                    last_sync_utc: row.get(2)?,
                    status: row.get(3)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
        })
    }
    pub fn log_event(
        &self,
        source_node_id: &str,
        event_type: &str,
        payload_json: &str,
        shared_secret: &str,
    ) -> Result<LoggedEvent, DbError> {
        let created_at_utc = utc_now();
        let canonical = format!(
            "{}|{}|{}|{}",
            source_node_id, event_type, payload_json, created_at_utc
        );
        let event_hash = sha256_hex(&canonical);
        let signature = sha256_hex(&format!("{}|{}", canonical, shared_secret));
        self.with_connection(|connection| {
            connection.execute(
                "INSERT INTO events (source_node_id, event_type, payload_json, event_hash, signature, created_at_utc)\n                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![source_node_id, event_type, payload_json, event_hash, signature, created_at_utc],
            )?;
            Ok(LoggedEvent {
                event_id: connection.last_insert_rowid(),
                event_hash,
                signature,
                created_at_utc,
            })
        })
    }
    pub fn get_status(&self) -> Result<StatusSnapshot, DbError> {
        self.with_connection(|connection| {
            let node_count = scalar_count(connection, "SELECT COUNT(*) FROM nodes")?;
            let mission_record_count = scalar_count(connection, "SELECT COUNT(*) FROM mission_records")?;
            let event_count = scalar_count(connection, "SELECT COUNT(*) FROM events")?;
            let replication_peer_count = scalar_count(connection, "SELECT COUNT(*) FROM replication_state")?;
            let last_event_utc = connection
                .query_row(
                    "SELECT created_at_utc FROM events ORDER BY event_id DESC LIMIT 1",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .optional()?;
            Ok(StatusSnapshot {
                node_count,
                mission_record_count,
                event_count,
                replication_peer_count,
                last_event_utc,
            })
        })
    }
    fn with_connection<T>(&self, operation: impl FnOnce(&Connection) -> Result<T, DbError>) -> Result<T, DbError> {
        let connection = Connection::open(&self.db_path)?;
        connection.busy_timeout(Duration::from_secs(3))?;
        connection.pragma_update(None, "foreign_keys", true)?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        operation(&connection)
    }
}
fn scalar_count(connection: &Connection, sql: &str) -> Result<i64, DbError> {
    let count = connection.query_row(sql, [], |row| row.get::<_, i64>(0))?;
    Ok(count)
}
fn utc_now() -> String {
    Utc::now().to_rfc3339()
}
fn sha256_hex(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    fn test_config(db_path: PathBuf) -> AppConfig {
        AppConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            node_id: "node-test".to_string(),
            node_role: "primary".to_string(),
            public_endpoint: "http://127.0.0.1:8080".to_string(),
            db_path,
            shared_secret: "test-secret".to_string(),
            bootstrap_peers: vec!["http://peer-1:8080".to_string()],
        }
    }
    #[test]
    fn bootstrap_creates_schema_and_node_record() {
        let temp_dir = tempdir().expect("tempdir");
        let db_path = temp_dir.path().join("node-test.db");
        let config = test_config(db_path.clone());
        let database = Database::new(db_path);
        database.bootstrap(&config).expect("bootstrap should succeed");
        let status = database.get_status().expect("status should load");
        let nodes = database.list_nodes().expect("nodes should load");
        let replication = database
            .list_replication_state()
            .expect("replication state should load");
        assert_eq!(status.node_count, 1);
        assert_eq!(status.replication_peer_count, 1);
        assert_eq!(nodes[0].node_id, "node-test");
        assert_eq!(replication[0].status, "planned");
    }
    #[test]
    fn log_event_generates_hash_and_signature() {
        let temp_dir = tempdir().expect("tempdir");
        let db_path = temp_dir.path().join("node-test.db");
        let config = test_config(db_path.clone());
        let database = Database::new(db_path);
        database.bootstrap(&config).expect("bootstrap should succeed");
        let logged_event = database
            .log_event(
                &config.node_id,
                "mission.order.created",
                r#"{"order_id":"ORD-001"}"#,
                &config.shared_secret,
            )
            .expect("event should be inserted");
        let status = database.get_status().expect("status should load");
        assert_eq!(status.event_count, 1);
        assert_eq!(logged_event.event_hash.len(), 64);
        assert_eq!(logged_event.signature.len(), 64);
        assert!(status.last_event_utc.is_some());
    }
}
