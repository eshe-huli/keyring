//! Keyring membership — trust groups for scoped data sharing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::NodeId;

/// Unique keyring identifier
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct KeyringId(Uuid);

impl KeyringId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// The special __system__ keyring for coordination
    pub fn system() -> Self {
        // Deterministic UUID for system keyring
        Self(Uuid::from_bytes([
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00,
            0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ]))
    }
}

impl std::fmt::Display for KeyringId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Role within a keyring
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    /// Full access — read, write, invite, revoke
    Admin,
    /// Read and write
    Member,
    /// Read only
    Reader,
    /// Can only sync specific document types
    Restricted(Vec<String>),
}

/// A keyring membership definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyringMembership {
    pub id: KeyringId,
    pub name: String,
    pub members: HashMap<NodeId, Role>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl KeyringMembership {
    pub fn new(name: String, creator: NodeId) -> Self {
        let mut members = HashMap::new();
        members.insert(creator, Role::Admin);

        Self {
            id: KeyringId::new(),
            name,
            members,
            created_at: chrono::Utc::now(),
        }
    }

    pub fn invite(&mut self, node: NodeId, role: Role) {
        self.members.insert(node, role);
    }

    pub fn revoke(&mut self, node: &NodeId) {
        self.members.remove(node);
    }

    pub fn has_member(&self, node: &NodeId) -> bool {
        self.members.contains_key(node)
    }

    pub fn role_of(&self, node: &NodeId) -> Option<&Role> {
        self.members.get(node)
    }
}
