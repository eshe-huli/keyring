//! Content-addressed blobs — the fundamental storage unit

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::identity::NodeId;

/// BLAKE3 hash — content address
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlobHash([u8; 32]);

impl BlobHash {
    pub fn compute(data: &[u8]) -> Self {
        let hash = blake3::hash(data);
        Self(*hash.as_bytes())
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn short(&self) -> String {
        hex::encode(&self.0[..8])
    }
}

impl fmt::Display for BlobHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl fmt::Debug for BlobHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash({})", self.short())
    }
}

impl AsRef<[u8]> for BlobHash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// A content-addressed blob
#[derive(Clone, Serialize, Deserialize)]
pub struct Blob {
    pub hash: BlobHash,
    pub data: Vec<u8>,
    pub size: u64,
    pub created: chrono::DateTime<chrono::Utc>,
    pub author: NodeId,
}

impl Blob {
    pub fn new(data: Vec<u8>, author: NodeId) -> Self {
        let hash = BlobHash::compute(&data);
        let size = data.len() as u64;

        Self {
            hash,
            data,
            size,
            created: chrono::Utc::now(),
            author,
        }
    }

    /// Verify blob integrity
    pub fn verify(&self) -> bool {
        BlobHash::compute(&self.data) == self.hash
    }
}
