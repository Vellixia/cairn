//! The memory engine: persist what matters and surface it again across sessions.
//!
//! This is the thin-slice version: dedup on exact content, and recall ranked by keyword overlap +
//! recency + importance. Hybrid retrieval (BM25 + vectors + graph, RRF) and 4-tier consolidation
//! land in later phases; the API here is designed to stay stable as those arrive.

use cairn_core::{ContentHash, Memory, MemoryKind, NewMemory, Result};
use cairn_store::Store;
use chrono::Utc;
use serde::Serialize;
use std::sync::Arc;

/// A recall hit with its relevance score.
#[derive(Debug, Clone, Serialize)]
pub struct ScoredMemory {
    pub memory: Memory,
    pub score: f32,
}

pub struct MemoryEngine {
    store: Arc<Store>,
}

impl MemoryEngine {
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }

    /// Persist a memory. If an identical one already exists, return it instead of duplicating.
    pub fn remember(&self, input: NewMemory) -> Result<Memory> {
        let memory = input.into_memory();
        let hash = ContentHash::of_str(&memory.content);
        if let Some(existing) = self.store.find_memory_by_content_hash(hash.as_str())? {
            return Ok(existing);
        }
        self.store.insert_memory(&memory)?;
        Ok(memory)
    }

    /// Recall the most relevant memories for a query.
    pub fn recall(&self, query: &str, limit: usize) -> Result<Vec<ScoredMemory>> {
        let terms: Vec<String> = query
            .to_lowercase()
            .split_whitespace()
            .map(str::to_string)
            .collect();
        let now = Utc::now();

        let mut scored: Vec<ScoredMemory> = self
            .store
            .all_memories()?
            .into_iter()
            .map(|m| {
                let haystack = format!(
                    "{} {}",
                    m.content.to_lowercase(),
                    m.concepts.join(" ").to_lowercase()
                );
                let overlap = terms
                    .iter()
                    .filter(|t| haystack.contains(t.as_str()))
                    .count() as f32;
                let age_days = ((now - m.created_at).num_seconds() as f32 / 86_400.0).max(0.0);
                let recency = 1.0 / (1.0 + age_days);
                let score = overlap * 2.0 + m.importance + recency * 0.5;
                ScoredMemory { memory: m, score }
            })
            .collect();

        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(limit);

        for s in &scored {
            let _ = self.store.touch_memory(&s.memory.id);
        }
        Ok(scored)
    }

    /// The session-start bootstrap: the highest-value memories to inject so the agent never
    /// starts cold. Prioritizes decisions/tasks/preferences, then importance and recency.
    pub fn wakeup(&self, limit: usize) -> Result<Vec<Memory>> {
        let now = Utc::now();
        let mut all = self.store.all_memories()?;
        all.sort_by(|a, b| {
            priority(a, now)
                .partial_cmp(&priority(b, now))
                .unwrap_or(std::cmp::Ordering::Equal)
                .reverse()
        });
        all.truncate(limit);
        Ok(all)
    }
}

fn priority(m: &Memory, now: chrono::DateTime<Utc>) -> f32 {
    let kind_weight = match m.kind {
        MemoryKind::Decision => 1.0,
        MemoryKind::Task => 0.9,
        MemoryKind::Preference => 0.8,
        MemoryKind::Gotcha => 0.7,
        MemoryKind::Fact => 0.5,
        MemoryKind::Note => 0.3,
    };
    let age_days = ((now - m.created_at).num_seconds() as f32 / 86_400.0).max(0.0);
    let recency = 1.0 / (1.0 + age_days);
    kind_weight + m.importance + recency * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;
    use cairn_core::{Config, MemoryKind};
    use cairn_store::Store;

    fn engine() -> (MemoryEngine, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let cfg = Config::resolve(Some(dir.path().join("data"))).unwrap();
        let store = Arc::new(Store::open(&cfg).unwrap());
        (MemoryEngine::new(store), dir)
    }

    #[test]
    fn identical_content_dedups() {
        let (mem, _d) = engine();
        let a = mem
            .remember(NewMemory::new("use sqlite for storage"))
            .unwrap();
        let b = mem
            .remember(NewMemory::new("use sqlite for storage"))
            .unwrap();
        assert_eq!(
            a.id, b.id,
            "identical content must dedup to the same memory"
        );
    }

    #[test]
    fn recall_ranks_relevant_first() {
        let (mem, _d) = engine();
        mem.remember(NewMemory::new("use sqlite plus a content-hash blob store"))
            .unwrap();
        mem.remember(NewMemory::new("the weather today is sunny"))
            .unwrap();
        let hits = mem.recall("sqlite blob storage", 10).unwrap();
        assert!(!hits.is_empty());
        assert!(hits[0].memory.content.contains("sqlite"));
    }

    #[test]
    fn wakeup_prioritizes_decisions() {
        let (mem, _d) = engine();
        mem.remember(NewMemory::new("a passing note")).unwrap();
        mem.remember(NewMemory {
            content: "decided to build the engine in Rust".into(),
            kind: Some(MemoryKind::Decision),
            importance: Some(0.9),
            ..Default::default()
        })
        .unwrap();
        let w = mem.wakeup(5).unwrap();
        assert_eq!(w[0].kind, MemoryKind::Decision);
    }
}
