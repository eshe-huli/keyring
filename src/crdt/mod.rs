//! CRDT engine — conflict-free replicated data types
//!
//! Built on Automerge for document CRDTs.
//! Additional types: LWW-Register, OR-Set, PN-Counter, Causal Queue

use serde::{Deserialize, Serialize};

use crate::identity::NodeId;
use crate::store::blob::BlobHash;

/// Hybrid Logical Clock — captures causality across nodes
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Hlc {
    /// Physical time (milliseconds since epoch)
    pub physical: u64,
    /// Logical counter (monotonically increasing)
    pub logical: u32,
    /// Node that generated this timestamp
    pub node: NodeId,
}

impl Hlc {
    pub fn now(node: NodeId) -> Self {
        let physical = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            physical,
            logical: 0,
            node,
        }
    }

    /// Merge with a received HLC (take max + increment)
    pub fn merge(&self, other: &Hlc, node: NodeId) -> Self {
        let physical = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        if physical > self.physical && physical > other.physical {
            Hlc { physical, logical: 0, node }
        } else if self.physical == other.physical {
            Hlc {
                physical: self.physical,
                logical: self.logical.max(other.logical) + 1,
                node,
            }
        } else if self.physical > other.physical {
            Hlc {
                physical: self.physical,
                logical: self.logical + 1,
                node,
            }
        } else {
            Hlc {
                physical: other.physical,
                logical: other.logical + 1,
                node,
            }
        }
    }
}

/// A change in the Merkle DAG
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Change {
    pub hash: BlobHash,
    pub parents: Vec<BlobHash>,
    pub author: NodeId,
    pub timestamp: Hlc,
    pub operation: Vec<u8>, // Serialized CRDT operation
    pub signature: Vec<u8>, // Ed25519 signature
}

/// Vector clock for causal ordering
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct VectorClock {
    pub clocks: std::collections::HashMap<NodeId, u64>,
}

impl VectorClock {
    pub fn increment(&mut self, node: NodeId) {
        let counter = self.clocks.entry(node).or_insert(0);
        *counter += 1;
    }

    pub fn merge(&mut self, other: &VectorClock) {
        for (node, &count) in &other.clocks {
            let entry = self.clocks.entry(*node).or_insert(0);
            *entry = (*entry).max(count);
        }
    }

    /// True if self happened before other
    pub fn happened_before(&self, other: &VectorClock) -> bool {
        let mut dominated = false;
        for (node, &count) in &other.clocks {
            let self_count = self.clocks.get(node).copied().unwrap_or(0);
            if self_count > count {
                return false;
            }
            if self_count < count {
                dominated = true;
            }
        }
        // Check we don't have entries they don't
        for (node, &count) in &self.clocks {
            if count > other.clocks.get(node).copied().unwrap_or(0) {
                return false;
            }
        }
        dominated
    }
}
