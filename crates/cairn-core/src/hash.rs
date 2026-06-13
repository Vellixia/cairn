//! Content-addressed hashing.
//!
//! Cairn's "no context loss" guarantee rests on this: whenever we compress a file, shell output,
//! response, or memory into a smaller *view*, we keep the full-fidelity original in the blob
//! store addressed by its [`ContentHash`]. Any view can therefore be expanded back to the
//! byte-identical original on demand.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A SHA-256 content hash, hex-encoded.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash(pub String);

impl ContentHash {
    /// Hash arbitrary bytes.
    pub fn of(bytes: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        ContentHash(hex::encode(hasher.finalize()))
    }

    /// Hash a string's UTF-8 bytes.
    pub fn of_str(s: &str) -> Self {
        Self::of(s.as_bytes())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Short prefix, handy for display/handles.
    pub fn short(&self) -> &str {
        &self.0[..self.0.len().min(12)]
    }
}

impl std::fmt::Display for ContentHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_and_distinct() {
        let a = ContentHash::of_str("hello");
        let b = ContentHash::of_str("hello");
        let c = ContentHash::of_str("world");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.0.len(), 64); // sha256 hex
        assert_eq!(a.short().len(), 12);
    }
}
