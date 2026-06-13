//! SQLite-backed structured store (memories, …) plus access to the blob store.

use crate::blob::BlobStore;
use cairn_core::{Config, ContentHash, Error, Memory, MemoryKind, MemoryTier, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::sync::Mutex;

/// Map any storage-backend error into the shared error type.
fn stor<E: std::fmt::Display>(e: E) -> Error {
    Error::Storage(e.to_string())
}

const SELECT_COLS: &str = "id,kind,tier,content,content_hash,concepts,files,session_id,importance,access_count,created_at,updated_at";

pub struct Store {
    conn: Mutex<Connection>,
    blobs: BlobStore,
}

impl Store {
    /// Open (and migrate) the database described by `cfg`.
    pub fn open(cfg: &Config) -> Result<Self> {
        let conn = Connection::open(cfg.db_path()).map_err(stor)?;
        let store = Self {
            conn: Mutex::new(conn),
            blobs: BlobStore::new(cfg.blobs_dir()),
        };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS memories (
                id           TEXT PRIMARY KEY,
                kind         TEXT NOT NULL,
                tier         TEXT NOT NULL,
                content      TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                concepts     TEXT NOT NULL,
                files        TEXT NOT NULL,
                session_id   TEXT,
                importance   REAL NOT NULL,
                access_count INTEGER NOT NULL DEFAULT 0,
                created_at   TEXT NOT NULL,
                updated_at   TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_memories_hash ON memories(content_hash);
            CREATE INDEX IF NOT EXISTS idx_memories_created ON memories(created_at);",
        )
        .map_err(stor)?;
        Ok(())
    }

    pub fn blobs(&self) -> &BlobStore {
        &self.blobs
    }

    pub fn insert_memory(&self, m: &Memory) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let hash = ContentHash::of_str(&m.content);
        conn.execute(
            "INSERT INTO memories (id,kind,tier,content,content_hash,concepts,files,session_id,importance,access_count,created_at,updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
            params![
                m.id,
                m.kind.as_str(),
                m.tier.as_str(),
                m.content,
                hash.as_str(),
                serde_json::to_string(&m.concepts)?,
                serde_json::to_string(&m.files)?,
                m.session_id,
                m.importance,
                m.access_count,
                m.created_at.to_rfc3339(),
                m.updated_at.to_rfc3339(),
            ],
        )
        .map_err(stor)?;
        Ok(())
    }

    pub fn get_memory(&self, id: &str) -> Result<Option<Memory>> {
        let conn = self.conn.lock().unwrap();
        let sql = format!("SELECT {SELECT_COLS} FROM memories WHERE id=?1");
        let mut stmt = conn.prepare(&sql).map_err(stor)?;
        stmt.query_row(params![id], row_to_memory)
            .optional()
            .map_err(stor)
    }

    pub fn find_memory_by_content_hash(&self, hash: &str) -> Result<Option<Memory>> {
        let conn = self.conn.lock().unwrap();
        let sql = format!("SELECT {SELECT_COLS} FROM memories WHERE content_hash=?1 LIMIT 1");
        let mut stmt = conn.prepare(&sql).map_err(stor)?;
        stmt.query_row(params![hash], row_to_memory)
            .optional()
            .map_err(stor)
    }

    pub fn all_memories(&self) -> Result<Vec<Memory>> {
        let conn = self.conn.lock().unwrap();
        let sql = format!("SELECT {SELECT_COLS} FROM memories ORDER BY created_at DESC");
        let mut stmt = conn.prepare(&sql).map_err(stor)?;
        let rows = stmt.query_map([], row_to_memory).map_err(stor)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(stor)?);
        }
        Ok(out)
    }

    pub fn touch_memory(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE memories SET access_count = access_count + 1, updated_at = ?2 WHERE id = ?1",
            params![id, Utc::now().to_rfc3339()],
        )
        .map_err(stor)?;
        Ok(())
    }

    pub fn count_memories(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT COUNT(*) FROM memories", [], |r| r.get(0))
            .map_err(stor)
    }
}

fn row_to_memory(row: &rusqlite::Row) -> rusqlite::Result<Memory> {
    let kind: String = row.get("kind")?;
    let tier: String = row.get("tier")?;
    let concepts: String = row.get("concepts")?;
    let files: String = row.get("files")?;
    let created: String = row.get("created_at")?;
    let updated: String = row.get("updated_at")?;
    Ok(Memory {
        id: row.get("id")?,
        kind: kind.parse().unwrap_or(MemoryKind::Note),
        tier: tier.parse().unwrap_or(MemoryTier::Working),
        content: row.get("content")?,
        concepts: serde_json::from_str(&concepts).unwrap_or_default(),
        files: serde_json::from_str(&files).unwrap_or_default(),
        session_id: row.get("session_id")?,
        importance: row.get("importance")?,
        access_count: row.get("access_count")?,
        created_at: parse_ts(&created),
        updated_at: parse_ts(&updated),
    })
}

fn parse_ts(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}
