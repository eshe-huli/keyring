//! ContentStore — redb-backed content-addressed storage
//!
//! Tables:
//!   blobs:      BLAKE3Hash → Bytes
//!   documents:  DocumentId → DocumentMeta
//!   sync_state: NodeId → SyncCursor

use anyhow::Result;
use redb::{Database, ReadableTable, TableDefinition};
use std::path::Path;
use std::sync::Arc;

use super::blob::{Blob, BlobHash};
use super::document::{Document, DocumentId, DocumentMeta};
use crate::identity::KeyringId;

const BLOBS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("blobs");
const DOCUMENTS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("documents");
const DOC_DATA: TableDefinition<&[u8], &[u8]> = TableDefinition::new("doc_data");

pub struct ContentStore {
    db: Arc<Database>,
}

impl ContentStore {
    /// Open or create a content store at the given path
    pub fn open(path: &Path) -> Result<Self> {
        let db_path = path.join("keyring.redb");

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db = Database::create(&db_path)?;

        // Initialize tables
        let write_txn = db.begin_write()?;
        {
            let _ = write_txn.open_table(BLOBS)?;
            let _ = write_txn.open_table(DOCUMENTS)?;
            let _ = write_txn.open_table(DOC_DATA)?;
        }
        write_txn.commit()?;

        Ok(Self { db: Arc::new(db) })
    }

    // ─── Blob operations ───

    /// Store a blob (content-addressed, deduplicated)
    pub fn put_blob(&self, blob: &Blob) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(BLOBS)?;
            table.insert(blob.hash.as_bytes().as_slice(), blob.data.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Retrieve a blob by hash
    pub fn get_blob(&self, hash: &BlobHash) -> Result<Option<Vec<u8>>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(BLOBS)?;

        match table.get(hash.as_bytes().as_slice())? {
            Some(data) => Ok(Some(data.value().to_vec())),
            None => Ok(None),
        }
    }

    /// Check if a blob exists
    pub fn has_blob(&self, hash: &BlobHash) -> Result<bool> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(BLOBS)?;
        Ok(table.get(hash.as_bytes().as_slice())?.is_some())
    }

    // ─── Document operations ───

    /// Store a document (metadata + CRDT state)
    pub fn put_document(&self, doc: &Document) -> Result<()> {
        let meta_bytes = bincode::serialize(&doc.meta)?;
        let id_bytes = bincode::serialize(&doc.meta.id)?;

        let write_txn = self.db.begin_write()?;
        {
            let mut meta_table = write_txn.open_table(DOCUMENTS)?;
            meta_table.insert(id_bytes.as_slice(), meta_bytes.as_slice())?;

            let mut data_table = write_txn.open_table(DOC_DATA)?;
            data_table.insert(id_bytes.as_slice(), doc.crdt_state.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Get a document by ID
    pub fn get_document(&self, id: &DocumentId) -> Result<Option<Document>> {
        let id_bytes = bincode::serialize(id)?;
        let read_txn = self.db.begin_read()?;

        let meta_table = read_txn.open_table(DOCUMENTS)?;
        let data_table = read_txn.open_table(DOC_DATA)?;

        let meta = match meta_table.get(id_bytes.as_slice())? {
            Some(data) => bincode::deserialize::<DocumentMeta>(data.value())?,
            None => return Ok(None),
        };

        let crdt_state = match data_table.get(id_bytes.as_slice())? {
            Some(data) => data.value().to_vec(),
            None => vec![],
        };

        Ok(Some(Document { meta, crdt_state }))
    }

    /// List all documents in a keyring
    pub fn list_documents(&self, keyring: &KeyringId) -> Result<Vec<DocumentMeta>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(DOCUMENTS)?;

        let mut docs = Vec::new();
        for entry in table.iter()? {
            let (_, value) = entry?;
            let meta: DocumentMeta = bincode::deserialize(value.value())?;
            if meta.keyring == *keyring {
                docs.push(meta);
            }
        }

        Ok(docs)
    }

    /// Delete a document
    pub fn delete_document(&self, id: &DocumentId) -> Result<bool> {
        let id_bytes = bincode::serialize(id)?;
        let write_txn = self.db.begin_write()?;
        let existed;
        {
            let mut meta_table = write_txn.open_table(DOCUMENTS)?;
            existed = meta_table.remove(id_bytes.as_slice())?.is_some();

            let mut data_table = write_txn.open_table(DOC_DATA)?;
            data_table.remove(id_bytes.as_slice())?;
        }
        write_txn.commit()?;
        Ok(existed)
    }
}
