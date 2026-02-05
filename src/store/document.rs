//! Documents — CRDT-enabled, versioned, structured data units

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::identity::{KeyringId, NodeId};

/// Stable document identifier
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct DocumentId(Uuid);

impl DocumentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::fmt::Display for DocumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Document type determines CRDT strategy
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentType {
    /// Text/binary file — Automerge for text, LWW for binary
    File,
    /// Append-only event log
    Event,
    /// Last-write-wins register (settings, status)
    Register,
    /// Increment/decrement counter
    Counter,
    /// Add/remove set (tags, memberships)
    Set,
    /// Key-value map
    Map,
    /// Task queue
    Queue,
}

/// Document metadata (stored in redb index)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocumentMeta {
    pub id: DocumentId,
    pub doc_type: DocumentType,
    pub keyring: KeyringId,
    pub tags: HashSet<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub author: NodeId,
    pub size: u64,
}

/// A full document with CRDT state
#[derive(Clone)]
pub struct Document {
    pub meta: DocumentMeta,
    /// Raw Automerge document bytes (serialized CRDT state)
    pub crdt_state: Vec<u8>,
}

impl Document {
    pub fn new_register(keyring: KeyringId, author: NodeId, value: &[u8]) -> Self {
        let id = DocumentId::new();
        let now = chrono::Utc::now();

        Self {
            meta: DocumentMeta {
                id,
                doc_type: DocumentType::Register,
                keyring,
                tags: HashSet::new(),
                metadata: HashMap::new(),
                created_at: now,
                updated_at: now,
                author,
                size: value.len() as u64,
            },
            crdt_state: value.to_vec(),
        }
    }

    pub fn new_event(keyring: KeyringId, author: NodeId, payload: &[u8]) -> Self {
        let id = DocumentId::new();
        let now = chrono::Utc::now();

        Self {
            meta: DocumentMeta {
                id,
                doc_type: DocumentType::Event,
                keyring,
                tags: HashSet::new(),
                metadata: HashMap::new(),
                created_at: now,
                updated_at: now,
                author,
                size: payload.len() as u64,
            },
            crdt_state: payload.to_vec(),
        }
    }
}
