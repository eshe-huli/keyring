//! Content-addressed storage — BLAKE3 hashing, redb backend, Merkle DAG

pub mod blob;
pub mod content_store;
pub mod document;

pub use blob::{Blob, BlobHash};
pub use content_store::ContentStore;
pub use document::{Document, DocumentId, DocumentType, DocumentMeta};
