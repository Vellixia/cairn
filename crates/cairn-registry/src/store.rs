//! On-disk pack registry store. See [`crate`] for the layout and HTTP API surface.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};
use thiserror::Error;

use cairn_pack::{Manifest, PublicKey, VerifyError};

/// Errors that the registry can return to the HTTP layer. The [`From`] impls let callers
/// `?`-propagate `io::Error` and `serde_json::Error` cleanly.
#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("pack format: {0}")]
    Pack(String),
    #[error("Ed25519 signature did not match any trusted key")]
    InvalidSignature,
    #[error("not found: {0}")]
    NotFound(String),
    #[error("already exists: {0}")]
    AlreadyExists(String),
    #[error("malformed trusted key: {0}")]
    BadKey(String),
}

/// What `POST /registry/packs` returns — captures both the verification path taken and
/// the stored metadata, so the publisher can confirm where their pack ended up.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishReceipt {
    pub pack_id: String,
    pub name: String,
    pub version: String,
    pub signed_by: Option<String>, // hex pubkey if a trusted signer was matched
    pub status: PublishStatus,
    pub stored_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PublishStatus {
    /// Pack carried a `signature.ed25519` entry AND it verified against one of the
    /// trusted keys.
    Signed,
    /// Pack had no `signature.ed25519` entry. Stored because integrity hashes still
    /// match — but no author authenticity is asserted. The CLI / federation layer is
    /// responsible for warning the user when this happens.
    Unsigned,
}

/// What we keep in `index.json` — the fields a `GET /registry/packs` caller wants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackMeta {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub stored_at: DateTime<Utc>,
    pub size_bytes: u64,
    /// Hex-encoded author pubkey if this pack was signed by a trusted signer at
    /// publish time. `None` for unsigned packs.
    pub signer_pubkey: Option<String>,
    pub has_ed25519_signature: bool,
    pub memory_count: usize,
    pub download_count: u64,
}

/// Append-only log entry recording that an operator (or federation peer) revoked a
/// pack version. The federation sync layer (Sprint 14) replays these to subscribers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevocationEvent {
    pub name: String,
    pub version: String,
    pub revoked_at: DateTime<Utc>,
    pub reason: Option<String>,
}

/// Registry's only mutable state. Cheap to wrap in `Arc` — the disk is the source of
/// truth, this is a coarse write lock to keep index/keys/revocations consistent.
pub struct Registry {
    root: PathBuf,
    state: Mutex<RegistryState>,
}

#[derive(Default)]
struct RegistryState {
    index: Vec<PackMeta>,
    trusted_keys: Vec<PublicKey>,
    revocations: Vec<RevocationEvent>,
}

impl Registry {
    /// Open (or create) a registry rooted at `<data_dir>/registry/`. If the directory
    /// doesn't exist, it's created. Existing `index.json`, `trusted_keys.json`, and
    /// `revocations.jsonl` are loaded if present.
    pub fn open(data_dir: &Path) -> Result<Self, RegistryError> {
        let root = data_dir.join("registry");
        fs::create_dir_all(root.join("packs"))?;
        let index: Vec<PackMeta> = match fs::read(root.join("index.json")) {
            Ok(b) => serde_json::from_slice(&b)?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(e.into()),
        };
        let trusted_keys: Vec<PublicKey> = match fs::read(root.join("trusted_keys.json")) {
            Ok(b) => serde_json::from_slice(&b).unwrap_or_default(),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(e.into()),
        };
        let revocations: Vec<RevocationEvent> = match fs::read(root.join("revocations.jsonl")) {
            Ok(b) => b
                .split(|c| *c == b'\n')
                .filter(|l| !l.is_empty())
                .map(serde_json::from_slice)
                .collect::<Result<Vec<_>, _>>()?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(e.into()),
        };
        Ok(Self {
            root,
            state: Mutex::new(RegistryState {
                index,
                trusted_keys,
                revocations,
            }),
        })
    }

    /// List all packs, newest first.
    pub fn list_all(&self) -> Result<Vec<PackMeta>, RegistryError> {
        let g = self.state.lock().expect("registry lock poisoned");
        let mut out = g.index.clone();
        out.sort_by_key(|m| std::cmp::Reverse(m.stored_at));
        Ok(out)
    }

    /// List versions of a single pack (any order — caller can sort by `created_at`).
    pub fn list_versions(&self, name: &str) -> Result<Vec<PackMeta>, RegistryError> {
        let g = self.state.lock().expect("registry lock poisoned");
        Ok(g.index.iter().filter(|m| m.name == name).cloned().collect())
    }

    /// Search case-insensitively across `name + description + author`.
    pub fn search(&self, query: &str) -> Result<Vec<PackMeta>, RegistryError> {
        let q = query.to_ascii_lowercase();
        if q.is_empty() {
            return self.list_all();
        }
        let g = self.state.lock().expect("registry lock poisoned");
        Ok(g.index
            .iter()
            .filter(|m| {
                m.name.to_ascii_lowercase().contains(&q)
                    || m.description.to_ascii_lowercase().contains(&q)
                    || m.author.to_ascii_lowercase().contains(&q)
            })
            .cloned()
            .collect())
    }

    /// Return the trusted author public keys this registry will accept signatures from.
    pub fn trusted_keys(&self) -> Result<Vec<PublicKey>, RegistryError> {
        Ok(self
            .state
            .lock()
            .expect("registry lock poisoned")
            .trusted_keys
            .clone())
    }

    /// Add (or replace) a trusted public key. Used by the CLI's `pack trust <hex>` command
    /// or by the registry's first-run admin bootstrap.
    pub fn add_trusted_key(&self, key: PublicKey) -> Result<(), RegistryError> {
        let mut g = self.state.lock().expect("registry lock poisoned");
        if !g.trusted_keys.contains(&key) {
            g.trusted_keys.push(key);
            self.write_trusted_keys(&g.trusted_keys)?;
        }
        Ok(())
    }

    /// List the revocation log (chronological order).
    pub fn list_revocations(&self) -> Result<Vec<RevocationEvent>, RegistryError> {
        Ok(self
            .state
            .lock()
            .expect("registry lock poisoned")
            .revocations
            .clone())
    }

    /// Publish a tarball. Returns the receipt (including which path: signed/unsigned).
    ///
    /// **Verification policy:** if the tarball contains `signature.ed25519`, at least one
    /// of `trusted_keys` (or the per-call override) must verify the signature. If the
    /// tarball has no Ed25519 signature, it's stored anyway — the per-file SHA-256s are
    /// still integrity-checked at install time.
    pub fn publish(
        &self,
        tarball: &[u8],
        trusted_override: Option<&str>,
    ) -> Result<PublishReceipt, RegistryError> {
        // Parse the tarball to extract the manifest + signature entry.
        let entries = cairn_pack::tar(tarball).map_err(|e| RegistryError::Pack(e.to_string()))?;
        let manifest_entry = entries
            .iter()
            .find(|e| e.name == "manifest.json")
            .ok_or_else(|| RegistryError::Pack("missing manifest.json".into()))?;
        let manifest: Manifest =
            serde_json::from_slice(&manifest_entry.body).map_err(RegistryError::Json)?;

        // Decide which trusted-key set applies.
        let trusted = match trusted_override {
            Some(hex) => vec![parse_pubkey(hex)?],
            None => self
                .state
                .lock()
                .expect("registry lock poisoned")
                .trusted_keys
                .clone(),
        };

        let (status, signer_hex) = match cairn_pack::install::verify_ed25519_signature(
            &entries,
            &manifest_entry.body,
            &trusted,
        ) {
            Ok(true) => {
                // Find the matching key for the receipt — verify_ed25519 doesn't tell us which
                // key matched, so re-test to identify the signer.
                let signer = find_signer(&entries, &manifest_entry.body, &trusted);
                (PublishStatus::Signed, signer.map(|k| k.to_hex()))
            }
            Ok(false) => (PublishStatus::Unsigned, None),
            Err(VerifyError::Mismatch) => return Err(RegistryError::InvalidSignature),
            Err(_) => return Err(RegistryError::InvalidSignature),
        };

        // Stage the tarball to packs/<name>/<version>.cairnpkg.
        let pack_path = self
            .root
            .join("packs")
            .join(&manifest.name)
            .join(format!("{}.cairnpkg", manifest.version));
        if pack_path.exists() {
            return Err(RegistryError::AlreadyExists(format!(
                "{}-{}",
                manifest.name, manifest.version
            )));
        }
        if let Some(parent) = pack_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&pack_path, tarball)?;
        let size_bytes = fs::metadata(&pack_path)?.len();

        // Cache the manifest under a sibling filename so a quick `find` over the registry
        // can render metadata without unpacking.
        let manifest_cache = pack_path.with_file_name(format!(
            "{}.manifest.json",
            pack_path.file_name().unwrap().to_string_lossy()
        ));
        fs::write(&manifest_cache, serde_json::to_vec_pretty(&manifest)?)?;

        // Append to the index.
        let meta = PackMeta {
            id: manifest.id.clone(),
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            author: manifest.author.clone(),
            description: manifest.description.clone(),
            created_at: manifest.created_at,
            stored_at: Utc::now(),
            size_bytes,
            signer_pubkey: signer_hex.clone(),
            has_ed25519_signature: status == PublishStatus::Signed,
            memory_count: manifest.stats.memories,
            download_count: 0,
        };
        {
            let mut g = self.state.lock().expect("registry lock poisoned");
            g.index
                .retain(|m| !(m.name == meta.name && m.version == meta.version));
            g.index.push(meta.clone());
            self.write_index(&g.index)?;
        }

        Ok(PublishReceipt {
            pack_id: manifest.id,
            name: manifest.name,
            version: manifest.version,
            signed_by: signer_hex,
            status,
            stored_at: meta.stored_at,
        })
    }

    /// Read the raw tarball bytes for a stored pack.
    pub fn download_bytes(&self, name: &str, version: &str) -> Result<Vec<u8>, RegistryError> {
        let path = self
            .root
            .join("packs")
            .join(name)
            .join(format!("{version}.cairnpkg"));
        if !path.exists() {
            return Err(RegistryError::NotFound(format!("{name}-{version}")));
        }
        Ok(fs::read(&path)?)
    }

    /// Revoke (unpublish) a pack. Bumps the revocations log so federation peers see it.
    pub fn revoke(&self, name: &str, version: &str) -> Result<RevocationEvent, RegistryError> {
        let path = self
            .root
            .join("packs")
            .join(name)
            .join(format!("{version}.cairnpkg"));
        let manifest_path = self
            .root
            .join("packs")
            .join(name)
            .join(format!("{version}.manifest.json"));
        if !path.exists() {
            return Err(RegistryError::NotFound(format!("{name}-{version}")));
        }
        fs::remove_file(&path)?;
        let _ = fs::remove_file(&manifest_path);

        let event = RevocationEvent {
            name: name.to_string(),
            version: version.to_string(),
            revoked_at: Utc::now(),
            reason: None,
        };
        {
            let mut g = self.state.lock().expect("registry lock poisoned");
            g.index
                .retain(|m| !(m.name == name && m.version == version));
            self.write_index(&g.index)?;
            g.revocations.push(event.clone());
            self.write_revocations(&g.revocations)?;
        }
        Ok(event)
    }

    fn write_index(&self, index: &[PackMeta]) -> Result<(), RegistryError> {
        let tmp = self.root.join("index.json.tmp");
        {
            let mut f = fs::File::create(&tmp)?;
            let json = serde_json::to_vec_pretty(index)?;
            f.write_all(&json)?;
            f.sync_all()?;
        }
        fs::rename(tmp, self.root.join("index.json"))?;
        Ok(())
    }

    fn write_trusted_keys(&self, keys: &[PublicKey]) -> Result<(), RegistryError> {
        let tmp = self.root.join("trusted_keys.json.tmp");
        {
            let mut f = fs::File::create(&tmp)?;
            let json = serde_json::to_vec_pretty(keys)?;
            f.write_all(&json)?;
            f.sync_all()?;
        }
        fs::rename(tmp, self.root.join("trusted_keys.json"))?;
        Ok(())
    }

    fn write_revocations(&self, revs: &[RevocationEvent]) -> Result<(), RegistryError> {
        let tmp = self.root.join("revocations.jsonl.tmp");
        {
            let mut f = fs::File::create(&tmp)?;
            for r in revs {
                let json = serde_json::to_vec(r)?;
                f.write_all(&json)?;
                f.write_all(b"\n")?;
            }
            f.sync_all()?;
        }
        fs::rename(tmp, self.root.join("revocations.jsonl"))?;
        Ok(())
    }
}

/// Parse a hex-encoded 32-byte Ed25519 public key.
fn parse_pubkey(hex_str: &str) -> Result<PublicKey, RegistryError> {
    let bytes = hex::decode(hex_str.trim()).map_err(|_| RegistryError::BadKey(hex_str.into()))?;
    PublicKey::from_bytes(&bytes).map_err(|_| RegistryError::BadKey(hex_str.into()))
}

/// Find which trusted key signed this pack — used to populate the `signed_by` field on
/// the publish receipt.
fn find_signer(
    entries: &[cairn_pack::install::TarEntry],
    manifest_bytes: &[u8],
    trusted: &[PublicKey],
) -> Option<PublicKey> {
    for key in trusted {
        if cairn_pack::install::verify_ed25519_signature(entries, manifest_bytes, &[*key])
            .ok()
            .unwrap_or(false)
        {
            return Some(*key);
        }
    }
    None
}
#[cfg(test)]
mod tests {
    use super::*;
    use cairn_pack::{Keypair, Pack};
    use tempfile::TempDir;

    fn signed_pack_bytes(kp: &Keypair) -> Vec<u8> {
        let dir = TempDir::new().unwrap();
        let out = dir.path().join("p.cairnpkg");
        let mut pack = Pack::new("alpha", "1.0.0");
        pack.author = "tester".into();
        pack.description = "searchable text".into();
        pack.memories
            .push(serde_json::json!({"id": "m1", "content": "x"}));
        pack.write_tarball_signed(&out, kp).unwrap();
        std::fs::read(&out).unwrap()
    }

    #[test]
    fn publish_unsigned_pack_succeeds_and_is_listable() {
        let dir = TempDir::new().unwrap();
        let reg = Registry::open(dir.path()).unwrap();

        let bytes = {
            let td = TempDir::new().unwrap();
            let out = td.path().join("u.cairnpkg");
            let mut pack = Pack::new("plain", "0.1.0");
            pack.memories
                .push(serde_json::json!({"id": "m1", "content": "x"}));
            pack.write_tarball(&out).unwrap();
            std::fs::read(&out).unwrap()
        };
        let receipt = reg.publish(&bytes, None).unwrap();
        assert_eq!(receipt.status, PublishStatus::Unsigned);
        assert!(receipt.signed_by.is_none());

        let all = reg.list_all().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "plain");
        assert!(!all[0].has_ed25519_signature);
    }

    #[test]
    fn publish_signed_pack_with_trusted_key_returns_signed() {
        let dir = TempDir::new().unwrap();
        let reg = Registry::open(dir.path()).unwrap();
        let kp = Keypair::generate();
        reg.add_trusted_key(kp.public()).unwrap();

        let bytes = signed_pack_bytes(&kp);
        let receipt = reg.publish(&bytes, None).unwrap();
        assert_eq!(receipt.status, PublishStatus::Signed);
        assert_eq!(
            receipt.signed_by.as_deref(),
            Some(kp.public().to_hex().as_str())
        );
    }

    #[test]
    fn publish_signed_pack_without_trusted_key_rejects() {
        let dir = TempDir::new().unwrap();
        let reg = Registry::open(dir.path()).unwrap();
        // Note: no add_trusted_key call.
        let kp = Keypair::generate();
        let bytes = signed_pack_bytes(&kp);
        let err = reg.publish(&bytes, None).unwrap_err();
        assert!(
            matches!(err, RegistryError::InvalidSignature),
            "got {err:?}"
        );
    }

    #[test]
    fn publish_with_trust_override_accepts_only_the_overridden_key() {
        let dir = TempDir::new().unwrap();
        let reg = Registry::open(dir.path()).unwrap();
        let kp = Keypair::generate();
        let bytes = signed_pack_bytes(&kp);
        let receipt = reg.publish(&bytes, Some(&kp.public().to_hex())).unwrap();
        assert_eq!(receipt.status, PublishStatus::Signed);
    }

    #[test]
    fn download_bytes_round_trips_a_published_pack() {
        let dir = TempDir::new().unwrap();
        let reg = Registry::open(dir.path()).unwrap();
        let bytes = {
            let td = TempDir::new().unwrap();
            let out = td.path().join("r.cairnpkg");
            let mut pack = Pack::new("rt", "1.0.0");
            pack.memories
                .push(serde_json::json!({"id": "m1", "content": "x"}));
            pack.write_tarball(&out).unwrap();
            std::fs::read(&out).unwrap()
        };
        reg.publish(&bytes, None).unwrap();
        let downloaded = reg.download_bytes("rt", "1.0.0").unwrap();
        assert_eq!(downloaded, bytes);
    }

    #[test]
    fn revoke_removes_the_pack_and_appends_to_log() {
        let dir = TempDir::new().unwrap();
        let reg = Registry::open(dir.path()).unwrap();
        let bytes = {
            let td = TempDir::new().unwrap();
            let out = td.path().join("r.cairnpkg");
            let mut pack = Pack::new("killme", "1.0.0");
            pack.memories
                .push(serde_json::json!({"id": "m1", "content": "x"}));
            pack.write_tarball(&out).unwrap();
            std::fs::read(&out).unwrap()
        };
        reg.publish(&bytes, None).unwrap();
        let event = reg.revoke("killme", "1.0.0").unwrap();
        assert_eq!(event.name, "killme");
        assert_eq!(event.version, "1.0.0");

        // Subsequent download must fail.
        assert!(reg.download_bytes("killme", "1.0.0").is_err());
        // And the revocation log must include it.
        assert_eq!(reg.list_revocations().unwrap().len(), 1);
    }

    #[test]
    fn search_matches_name_description_and_author_case_insensitively() {
        let dir = TempDir::new().unwrap();
        let reg = Registry::open(dir.path()).unwrap();
        let kp = Keypair::generate();
        reg.add_trusted_key(kp.public()).unwrap();

        let by_name = signed_pack_bytes(&kp); // alpha, author "tester"
        let bytes2 = {
            let td = TempDir::new().unwrap();
            let out = td.path().join("z.cairnpkg");
            let mut pack = Pack::new("zebra-stripes", "2.0.0");
            pack.author = "another-author".into();
            pack.description = "all about horses".into();
            pack.memories
                .push(serde_json::json!({"id": "m1", "content": "x"}));
            pack.write_tarball(&out).unwrap();
            std::fs::read(&out).unwrap()
        };
        reg.publish(&by_name, None).unwrap();
        reg.publish(&bytes2, None).unwrap();

        let hits = reg.search("ALPHA").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].name, "alpha");
        let hits = reg.search("horses").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].name, "zebra-stripes");
        let hits = reg.search("tester").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].name, "alpha");
        let hits = reg.search("another-author").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].name, "zebra-stripes");
    }

    #[test]
    fn list_versions_returns_only_that_pack() {
        let dir = TempDir::new().unwrap();
        let reg = Registry::open(dir.path()).unwrap();
        for v in ["1.0.0", "1.1.0", "2.0.0"] {
            let td = TempDir::new().unwrap();
            let out = td.path().join(format!("p-{v}.cairnpkg"));
            let mut pack = Pack::new("multi", v);
            pack.memories
                .push(serde_json::json!({"id": "m1", "content": "x"}));
            pack.write_tarball(&out).unwrap();
            reg.publish(&std::fs::read(&out).unwrap(), None).unwrap();
        }
        let versions = reg.list_versions("multi").unwrap();
        assert_eq!(versions.len(), 3);
        let names: Vec<&str> = versions.iter().map(|v| v.version.as_str()).collect();
        assert!(names.contains(&"1.0.0"));
        assert!(names.contains(&"1.1.0"));
        assert!(names.contains(&"2.0.0"));
    }
}
