PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS nodes (
  node_id TEXT PRIMARY KEY,
  role TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'online',
  endpoint TEXT NOT NULL,
  metadata_json TEXT NOT NULL DEFAULT '{}',
  last_seen_utc TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS mission_records (
  record_id INTEGER PRIMARY KEY AUTOINCREMENT,
  record_type TEXT NOT NULL CHECK (record_type IN ('order', 'logistics', 'intelligence')),
  title TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  source_node_id TEXT NOT NULL,
  created_at_utc TEXT NOT NULL,
  updated_at_utc TEXT NOT NULL,
  FOREIGN KEY (source_node_id) REFERENCES nodes(node_id) ON DELETE RESTRICT
);

CREATE TABLE IF NOT EXISTS events (
  event_id INTEGER PRIMARY KEY AUTOINCREMENT,
  source_node_id TEXT NOT NULL,
  event_type TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  event_hash TEXT NOT NULL UNIQUE,
  signature TEXT NOT NULL,
  created_at_utc TEXT NOT NULL,
  FOREIGN KEY (source_node_id) REFERENCES nodes(node_id) ON DELETE RESTRICT
);

CREATE TABLE IF NOT EXISTS replication_state (
  peer_node_id TEXT PRIMARY KEY,
  last_replicated_event_id INTEGER NOT NULL DEFAULT 0,
  last_sync_utc TEXT,
  status TEXT NOT NULL DEFAULT 'pending'
);

CREATE INDEX IF NOT EXISTS idx_mission_records_type ON mission_records(record_type);
CREATE INDEX IF NOT EXISTS idx_events_source_node ON events(source_node_id, created_at_utc DESC);
CREATE INDEX IF NOT EXISTS idx_replication_state_status ON replication_state(status);

