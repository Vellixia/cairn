//! Content-addressed blob store on the filesystem.
//!
//! Files are stored under `blobs/<first-2-hex>/<full-hash>`, sharded to keep directories small.
//! Writes are idempotent (same content -> same path), so storing an original twice is free.

use cairn_core::{ContentHash, Result};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct BlobStore {
    root: PathBuf,
}

impl BlobStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn path_for(&self, hash: &ContentHash) -> PathBuf {
        let s = hash.as_str();
        self.root.join(&s[..2]).join(s)
    }

    /// Store bytes, returning their content hash. Idempotent.
    pub fn put(&self, bytes: &[u8]) -> Result<ContentHash> {
        let hash = ContentHash::of(bytes);
        let path = self.path_for(&hash);
        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, bytes)?;
        }
        Ok(hash)
    }

    pub fn put_str(&self, s: &str) -> Result<ContentHash> {
        self.put(s.as_bytes())
    }

    /// Fetch the exact original bytes for a hash, if present.
    pub fn get(&self, hash: &ContentHash) -> Result<Option<Vec<u8>>> {
        match fs::read(self.path_for(hash)) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_str(&self, hash: &ContentHash) -> Result<Option<String>> {
        Ok(self
            .get(hash)?
            .map(|b| String::from_utf8_lossy(&b).into_owned()))
    }

    pub fn has(&self, hash: &ContentHash) -> bool {
        self.path_for(hash).exists()
    }
}
