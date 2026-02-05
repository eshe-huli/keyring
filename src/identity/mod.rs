//! Identity module — Ed25519 keypair generation, node identity, keyrings
//!
//! Every node has a cryptographic identity generated at first boot.
//! NodeId = BLAKE3(public_key)

mod node;
mod keyring_id;

pub use node::{NodeId, NodeIdentity};
pub use keyring_id::{KeyringMembership, KeyringId};
