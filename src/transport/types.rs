//! Transport trait and common types

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use crate::identity::{KeyringId, NodeId};
use crate::store::document::DocumentId;

/// A peer address that can be resolved to a connection
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerAddr {
    pub node_id: Option<NodeId>,
    pub addr: String,
    pub transport: String,
}

/// Sync frame — the unit of communication between nodes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncFrame {
    pub frame_type: FrameType,
    pub keyring: KeyringId,
    pub document: Option<DocumentId>,
    pub payload: Vec<u8>,
    pub seq: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FrameType {
    /// Exchange Merkle roots to find divergence
    RootExchange,
    /// Request specific hashes for tree traversal
    HashRequest,
    /// Send delta changes
    DeltaPayload,
    /// Acknowledge receipt
    Ack,
    /// Presence announcement
    Presence,
    /// Ping/pong for health
    Ping,
    Pong,
}

/// An active connection to a peer
#[async_trait]
pub trait Connection: Send + Sync {
    async fn send(&self, frame: SyncFrame) -> Result<()>;
    async fn recv(&self) -> Result<SyncFrame>;
    fn peer_addr(&self) -> PeerAddr;
    async fn close(&self) -> Result<()>;
}

/// Listener for incoming connections
#[async_trait]
pub trait Listener: Send + Sync {
    async fn accept(&self) -> Result<Box<dyn Connection>>;
    fn local_addr(&self) -> Result<SocketAddr>;
}

/// The transport trait — all transports implement this
#[async_trait]
pub trait Transport: Send + Sync {
    async fn listen(&self, addr: &str) -> Result<Box<dyn Listener>>;
    async fn connect(&self, peer: &str) -> Result<Box<dyn Connection>>;
    async fn discover(&self) -> Result<Vec<PeerAddr>>;
    fn name(&self) -> &str;
}
