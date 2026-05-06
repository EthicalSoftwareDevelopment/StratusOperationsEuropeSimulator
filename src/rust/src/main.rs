use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::net::SocketAddr;
use stratus_foundation_service::{config::AppConfig, db::Database};
#[derive(Clone)]
struct AppState {
    config: AppConfig,
    database: Database,
}
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    node_id: String,
    role: String,
    db_path: String,
}
#[derive(Debug, Serialize)]
struct SecurityBaselineResponse {
    transport_encryption: &'static str,
    message_signing: &'static str,
    credential_scope: &'static str,
}
#[derive(Debug, Serialize)]
struct NetworkTopologyResponse {
    local_node_id: String,
    local_endpoint: String,
    bootstrap_peers: Vec<String>,
}
#[derive(Debug, Deserialize)]
struct NewEventRequest {
    event_type: String,
    payload: Value,
}
#[derive(Debug)]
enum AppError {
    BadRequest(String),
    Internal(String),
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message).into_response(),
            Self::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message).into_response(),
        }
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::from_env();
    let bind_address = config.bind_address();
    let database = Database::new(config.db_path.clone());
    database.bootstrap(&config)?;
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/status", get(status))
        .route("/api/nodes", get(nodes))
        .route("/api/network/topology", get(network_topology))
        .route("/api/security/baseline", get(security_baseline))
        .route("/api/events", post(create_event))
        .with_state(AppState { config, database });

    let listener = tokio::net::TcpListener::bind(&bind_address).await?;
    let local_addr: SocketAddr = listener.local_addr()?;
    println!(
        "Stratus Phase 1 foundation service listening on {}",
        local_addr
    );
    axum::serve(listener, app).await?;
    Ok(())
}
async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        node_id: state.config.node_id,
        role: state.config.node_role,
        db_path: state.database.path().display().to_string(),
    })
}
async fn status(State(state): State<AppState>) -> Result<Json<stratus_foundation_service::db::StatusSnapshot>, AppError> {
    state
        .database
        .get_status()
        .map(Json)
        .map_err(|error| AppError::Internal(error.to_string()))
}
async fn nodes(State(state): State<AppState>) -> Result<Json<Vec<stratus_foundation_service::db::NodeRecord>>, AppError> {
    state
        .database
        .list_nodes()
        .map(Json)
        .map_err(|error| AppError::Internal(error.to_string()))
}
async fn security_baseline() -> Json<SecurityBaselineResponse> {
    Json(SecurityBaselineResponse {
        transport_encryption: "planned via WireGuard and TLS in follow-on iterations",
        message_signing: "enabled for stored events using SHA-256 development signatures",
        credential_scope: "shared development secret; replace with per-node keys in Phase 1.3 hardening",
    })
}
async fn network_topology(State(state): State<AppState>) -> Json<NetworkTopologyResponse> {
    Json(NetworkTopologyResponse {
        local_node_id: state.config.node_id,
        local_endpoint: state.config.public_endpoint,
        bootstrap_peers: state.config.bootstrap_peers,
    })
}
async fn create_event(
    State(state): State<AppState>,
    Json(payload): Json<NewEventRequest>,
) -> Result<(StatusCode, Json<stratus_foundation_service::db::LoggedEvent>), AppError> {
    if payload.event_type.trim().is_empty() {
        return Err(AppError::BadRequest("event_type must not be empty".to_string()));
    }
    let payload_json = serde_json::to_string(&payload.payload)
        .map_err(|error| AppError::BadRequest(error.to_string()))?;
    let event = state
        .database
        .log_event(
            &state.config.node_id,
            payload.event_type.trim(),
            &payload_json,
            &state.config.shared_secret,
        )
        .map_err(|error| AppError::Internal(error.to_string()))?;
    Ok((StatusCode::CREATED, Json(event)))
}
