//! Sync engine — Merkle root exchange, delta computation, frame protocol
//!
//! Algorithm:
//! 1. Exchange Merkle roots for shared keyrings
//! 2. Binary search for divergence point
//! 3. Exchange only deltas
//! 4. CRDTs auto-converge
//! 5. Bandwidth = O(changes), not O(data)

use anyhow::Result;
use std::collections::HashMap;

use crate::identity::{KeyringId, NodeId};
use crate::store::blob::BlobHash;
use crate::store::document::DocumentId;

/// Sync cursor — tracks per-peer sync progress
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SyncCursor {
    pub node_id: NodeId,
    pub last_sync: chrono::DateTime<chrono::Utc>,
    pub roots: HashMap<DocumentId, BlobHash>,
}

/// Sync mode
#[derive(Clone, Debug)]
pub enum SyncMode {
    /// On local change — < 1 second
    Push,
    /// On reconnect — seconds
    Pull,
    /// Timer-based — minutes
    Periodic(std::time::Duration),
    /// User/agent command
    Manual,
    /// Random peer contact (large meshes)
    Gossip,
}

/// Sync manager — orchestrates sync across transports
pub struct SyncManager {
    mode: SyncMode,
    cursors: HashMap<NodeId, SyncCursor>,
}

impl SyncManager {
    pub fn new(mode: SyncMode) -> Self {
        Self {
            mode,
            cursors: HashMap::new(),
        }
    }

    /// Compute Merkle root for a document's change history
    pub fn compute_root(&self, _doc_id: &DocumentId) -> Result<BlobHash> {
        // TODO: Walk the Merkle DAG and compute root hash
        todo!("Merkle root computation")
    }

    /// Find divergent documents between local and remote roots
    pub fn find_divergence(
        &self,
        _local_roots: &HashMap<DocumentId, BlobHash>,
        _remote_roots: &HashMap<DocumentId, BlobHash>,
    ) -> Vec<DocumentId> {
        // TODO: Compare roots, return docs that differ
        todo!("Divergence detection")
    }

    /// Compute delta for a divergent document
    pub fn compute_delta(
        &self,
        _doc_id: &DocumentId,
        _remote_root: &BlobHash,
    ) -> Result<Vec<u8>> {
        // TODO: Binary search DAG, extract missing changes
        todo!("Delta computation")
    }

    /// Apply received delta to local store
    pub fn apply_delta(
        &self,
        _doc_id: &DocumentId,
        _delta: &[u8],
    ) -> Result<()> {
        // TODO: Deserialize changes, merge into CRDT, update DAG
        todo!("Delta application")
    }
}
