//! Node identity — Ed25519 keypair + BLAKE3 node ID

use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

/// 32-byte node identifier = BLAKE3(public_key)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId([u8; 32]);

impl NodeId {
    pub fn from_pubkey(pubkey: &VerifyingKey) -> Self {
        let hash = blake3::hash(pubkey.as_bytes());
        Self(*hash.as_bytes())
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn short(&self) -> String {
        hex::encode(&self.0[..8])
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl fmt::Debug for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeId({})", self.short())
    }
}

impl PartialOrd for NodeId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

/// Full node identity with signing capability
pub struct NodeIdentity {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    node_id: NodeId,
    name: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl NodeIdentity {
    /// Generate a new random identity
    pub fn generate(name: String) -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let node_id = NodeId::from_pubkey(&verifying_key);

        Self {
            signing_key,
            verifying_key,
            node_id,
            name,
            created_at: chrono::Utc::now(),
        }
    }

    /// Load identity from disk
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let data = std::fs::read(path)?;
        let stored: StoredIdentity = bincode::deserialize(&data)?;

        let signing_key = SigningKey::from_bytes(&stored.secret_key);
        let verifying_key = signing_key.verifying_key();
        let node_id = NodeId::from_pubkey(&verifying_key);

        Ok(Self {
            signing_key,
            verifying_key,
            node_id,
            name: stored.name,
            created_at: stored.created_at,
        })
    }

    /// Save identity to disk
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let stored = StoredIdentity {
            secret_key: self.signing_key.to_bytes(),
            name: self.name.clone(),
            created_at: self.created_at,
        };

        let data = bincode::serialize(&stored)?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, &data)?;

        // Restrict permissions (Unix)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }

    /// Sign arbitrary data
    pub fn sign(&self, data: &[u8]) -> Signature {
        self.signing_key.sign(data)
    }

    /// Verify a signature from another node
    pub fn verify(pubkey: &VerifyingKey, data: &[u8], sig: &Signature) -> bool {
        pubkey.verify(data, sig).is_ok()
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn pubkey(&self) -> &VerifyingKey {
        &self.verifying_key
    }

    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.created_at
    }
}

#[derive(Serialize, Deserialize)]
struct StoredIdentity {
    secret_key: [u8; 32],
    name: String,
    created_at: chrono::DateTime<chrono::Utc>,
}
