//! API layer — gRPC server + future HTTP/REST and Unix socket
//!
//! Day 1: gRPC skeleton
//! Future: HTTP/REST, Unix socket

use anyhow::Result;

/// API server configuration
pub struct ApiConfig {
    pub grpc_addr: String,
    pub http_addr: Option<String>,
    pub unix_socket: Option<String>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            grpc_addr: "[::]:4434".to_string(),
            http_addr: None,
            unix_socket: Some("/tmp/keyring.sock".to_string()),
        }
    }
}

/// Start the API server
pub async fn serve(_config: ApiConfig) -> Result<()> {
    // TODO: Start tonic gRPC server with Keyring service definition
    // For now, just log
    tracing::info!("API server starting (gRPC skeleton)");
    Ok(())
}
