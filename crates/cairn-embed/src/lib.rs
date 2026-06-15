//! Pluggable text embeddings — Cairn turns memory content into vectors so HelixDB can do semantic
//! (vector) recall alongside BM25.
//!
//! Three providers, chosen by [`cairn_core::EmbedConfig`] (`CAIRN_EMBED_PROVIDER`):
//! - **local** (default) — in-process `all-MiniLM-L6-v2` (384-dim) via `fastembed`/ONNX. No API
//!   key, nothing leaves the machine. Requires the `local` cargo feature.
//! - **openai** — `/v1/embeddings` (default `text-embedding-3-small`); needs `CAIRN_EMBED_API_KEY`.
//! - **ollama** — `/api/embed` (default `nomic-embed-text`) against a local Ollama server.

use cairn_core::{EmbedConfig, Error, Result};

/// Turns text into embedding vectors. `Send + Sync` so it can live in shared server state.
pub trait Embedder: Send + Sync {
    /// Embed a batch of texts, preserving order.
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
    /// The dimensionality of the vectors this embedder produces (the HelixDB vector index width).
    fn dim(&self) -> usize;
    /// Convenience: embed a single string.
    fn embed_one(&self, text: &str) -> Result<Vec<f32>> {
        let mut v = self.embed(std::slice::from_ref(&text.to_string()))?;
        Ok(v.pop().unwrap_or_default())
    }
}

/// Build the embedder described by `cfg` (provider + model + url + key).
pub fn from_config(cfg: &EmbedConfig) -> Result<Box<dyn Embedder>> {
    match cfg.provider.as_str() {
        "openai" => Ok(Box::new(OpenAiEmbedder::new(cfg)?)),
        "ollama" => Ok(Box::new(OllamaEmbedder::new(cfg))),
        "local" => local_embedder(cfg),
        "hashing" => Ok(Box::new(HashingEmbedder::new(cfg))),
        other => Err(Error::Invalid(format!(
            "unknown CAIRN_EMBED_PROVIDER '{other}' (use local | openai | ollama | hashing)"
        ))),
    }
}

#[cfg(feature = "local")]
fn local_embedder(cfg: &EmbedConfig) -> Result<Box<dyn Embedder>> {
    Ok(Box::new(local::LocalEmbedder::new(cfg)?))
}

#[cfg(not(feature = "local"))]
fn local_embedder(_cfg: &EmbedConfig) -> Result<Box<dyn Embedder>> {
    Err(Error::Invalid(
        "local embeddings need the `local` cargo feature; or set CAIRN_EMBED_PROVIDER=openai|ollama"
            .into(),
    ))
}

/// Cosine similarity of two equal-length vectors (1.0 = identical direction, 0.0 = orthogonal).
pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na * nb)
    }
}

// --- Hashing (deterministic, dependency-free) --------------------------------------------------

/// A deterministic embedder using the **hashing trick** (signed feature hashing of tokens,
/// L2-normalized). No model, no network, fully reproducible — texts that share tokens get higher
/// cosine similarity, so it preserves lexical relatedness. Ideal for tests and as a
/// zero-dependency fallback when the local model isn't compiled in (`CAIRN_EMBED_PROVIDER=hashing`;
/// `CAIRN_EMBED_MODEL` may set the dimension, default 384 to match all-MiniLM-L6-v2).
struct HashingEmbedder {
    dim: usize,
}

impl HashingEmbedder {
    fn new(cfg: &EmbedConfig) -> Self {
        let dim = cfg
            .model
            .as_deref()
            .and_then(|m| m.parse::<usize>().ok())
            .unwrap_or(384);
        Self { dim: dim.max(1) }
    }
}

impl Embedder for HashingEmbedder {
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(texts.iter().map(|t| hash_embed(t, self.dim)).collect())
    }
    fn dim(&self) -> usize {
        self.dim
    }
}

/// Signed feature-hashing of `text`'s tokens into a `dim`-length, L2-normalized vector.
fn hash_embed(text: &str, dim: usize) -> Vec<f32> {
    let mut v = vec![0f32; dim.max(1)];
    for token in text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
    {
        let h = fnv1a(&token.to_ascii_lowercase());
        let idx = (h % v.len() as u64) as usize;
        let sign = if (h >> 8) & 1 == 0 { 1.0 } else { -1.0 };
        v[idx] += sign;
    }
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in &mut v {
            *x /= norm;
        }
    }
    v
}

/// 64-bit FNV-1a hash.
fn fnv1a(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

// --- OpenAI ------------------------------------------------------------------------------------

struct OpenAiEmbedder {
    model: String,
    api_key: String,
    base: String,
    dim: usize,
}

impl OpenAiEmbedder {
    fn new(cfg: &EmbedConfig) -> Result<Self> {
        let api_key = cfg
            .api_key
            .clone()
            .ok_or_else(|| Error::Invalid("CAIRN_EMBED_API_KEY is required for openai".into()))?;
        let model = cfg
            .model
            .clone()
            .unwrap_or_else(|| "text-embedding-3-small".to_string());
        let base = cfg
            .url
            .clone()
            .unwrap_or_else(|| "https://api.openai.com".to_string());
        let dim = known_dim(&model).unwrap_or(1536);
        Ok(Self {
            model,
            api_key,
            base: base.trim_end_matches('/').to_string(),
            dim,
        })
    }
}

impl Embedder for OpenAiEmbedder {
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let resp: serde_json::Value = ureq::post(&format!("{}/v1/embeddings", self.base))
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .send_json(serde_json::json!({ "model": self.model, "input": texts }))
            .map_err(|e| Error::Other(format!("openai embeddings request: {e}")))?
            .into_json()
            .map_err(|e| Error::Other(format!("openai embeddings decode: {e}")))?;
        let data = resp
            .get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| Error::Other("openai embeddings: missing 'data'".into()))?;
        data.iter().map(|d| json_vec(d.get("embedding"))).collect()
    }
    fn dim(&self) -> usize {
        self.dim
    }
}

// --- Ollama ------------------------------------------------------------------------------------

struct OllamaEmbedder {
    model: String,
    base: String,
    dim: usize,
}

impl OllamaEmbedder {
    fn new(cfg: &EmbedConfig) -> Self {
        let model = cfg
            .model
            .clone()
            .unwrap_or_else(|| "nomic-embed-text".to_string());
        let base = cfg
            .url
            .clone()
            .unwrap_or_else(|| "http://localhost:11434".to_string());
        let dim = known_dim(&model).unwrap_or(768);
        Self {
            model,
            base: base.trim_end_matches('/').to_string(),
            dim,
        }
    }
}

impl Embedder for OllamaEmbedder {
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let resp: serde_json::Value = ureq::post(&format!("{}/api/embed", self.base))
            .send_json(serde_json::json!({ "model": self.model, "input": texts }))
            .map_err(|e| Error::Other(format!("ollama embeddings request: {e}")))?
            .into_json()
            .map_err(|e| Error::Other(format!("ollama embeddings decode: {e}")))?;
        let rows = resp
            .get("embeddings")
            .and_then(|e| e.as_array())
            .ok_or_else(|| Error::Other("ollama embeddings: missing 'embeddings'".into()))?;
        rows.iter().map(|r| json_vec(Some(r))).collect()
    }
    fn dim(&self) -> usize {
        self.dim
    }
}

/// Parse a JSON number array into `Vec<f32>`.
fn json_vec(v: Option<&serde_json::Value>) -> Result<Vec<f32>> {
    v.and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|n| n.as_f64())
                .map(|f| f as f32)
                .collect()
        })
        .ok_or_else(|| Error::Other("embedding response: expected a number array".into()))
}

/// Known output dimensions for common models (so the Helix vector index can be sized up front).
fn known_dim(model: &str) -> Option<usize> {
    match model {
        m if m.contains("text-embedding-3-large") => Some(3072),
        m if m.contains("text-embedding-3-small") || m.contains("ada-002") => Some(1536),
        m if m.contains("mxbai-embed-large") => Some(1024),
        m if m.contains("nomic-embed-text") => Some(768),
        m if m.contains("MiniLM-L6") || m.contains("all-minilm") => Some(384),
        _ => None,
    }
}

// --- Local (fastembed / ONNX) ------------------------------------------------------------------

#[cfg(feature = "local")]
mod local {
    use super::{EmbedConfig, Embedder, Error, Result};
    use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
    use std::sync::Mutex;

    /// In-process `all-MiniLM-L6-v2` (384-dim). The model is fetched once on first construction.
    pub struct LocalEmbedder {
        model: Mutex<TextEmbedding>,
        dim: usize,
    }

    impl LocalEmbedder {
        pub fn new(_cfg: &EmbedConfig) -> Result<Self> {
            let model = TextEmbedding::try_new(
                InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_show_download_progress(true),
            )
            .map_err(|e| Error::Other(format!("loading local embedding model: {e}")))?;
            Ok(Self {
                model: Mutex::new(model),
                dim: 384,
            })
        }
    }

    impl Embedder for LocalEmbedder {
        fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
            let docs: Vec<&str> = texts.iter().map(String::as_str).collect();
            self.model
                .lock()
                .unwrap()
                .embed(docs, None)
                .map_err(|e| Error::Other(format!("local embedding: {e}")))
        }
        fn dim(&self) -> usize {
            self.dim
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(provider: &str) -> EmbedConfig {
        EmbedConfig {
            provider: provider.to_string(),
            model: None,
            url: None,
            api_key: None,
        }
    }

    #[test]
    fn unknown_provider_is_rejected() {
        assert!(from_config(&cfg("magic")).is_err());
    }

    #[test]
    fn openai_requires_an_api_key() {
        assert!(from_config(&cfg("openai")).is_err());
        let mut c = cfg("openai");
        c.api_key = Some("sk-test".into());
        assert!(from_config(&c).is_ok());
    }

    #[test]
    fn ollama_builds_without_a_key_and_reports_its_dim() {
        let e = from_config(&cfg("ollama")).unwrap();
        assert_eq!(e.dim(), 768); // nomic-embed-text default
    }

    #[test]
    fn cosine_is_one_for_identical_and_zero_for_orthogonal() {
        let a = [1.0, 2.0, 3.0];
        assert!((cosine(&a, &a) - 1.0).abs() < 1e-6);
        assert!(cosine(&[1.0, 0.0], &[0.0, 1.0]).abs() < 1e-6);
        assert_eq!(cosine(&[0.0, 0.0], &[1.0, 1.0]), 0.0);
    }

    #[test]
    fn hashing_embedder_is_deterministic_and_preserves_lexical_similarity() {
        let e = from_config(&cfg("hashing")).unwrap();
        assert_eq!(e.dim(), 384);
        let v = e
            .embed(&[
                "use sqlite for the blob store".to_string(),
                "use sqlite blob storage".to_string(),
                "the weather today is sunny".to_string(),
            ])
            .unwrap();
        assert_eq!(v[0].len(), 384);
        // Deterministic.
        assert_eq!(e.embed_one("use sqlite for the blob store").unwrap(), v[0]);
        // Shared tokens -> higher cosine than an unrelated sentence.
        assert!(cosine(&v[0], &v[1]) > cosine(&v[0], &v[2]));
    }

    #[test]
    fn known_dim_maps_common_models() {
        assert_eq!(known_dim("text-embedding-3-small"), Some(1536));
        assert_eq!(known_dim("all-MiniLM-L6-v2"), Some(384));
        assert_eq!(known_dim("nomic-embed-text"), Some(768));
        assert_eq!(known_dim("mystery-model"), None);
    }

    // Real local-model test (downloads ~90MB on first run). Off by default; run with:
    //   cargo test -p cairn-embed --features local -- --ignored
    #[cfg(feature = "local")]
    #[test]
    #[ignore]
    fn local_embeds_384_dims_and_ranks_related_text_higher() {
        let e = from_config(&cfg("local")).unwrap();
        assert_eq!(e.dim(), 384);
        let q = e.embed_one("the cat sat on the warm mat").unwrap();
        let related = e.embed_one("a kitten is resting on a rug").unwrap();
        let unrelated = e.embed_one("quarterly stock prices fell sharply").unwrap();
        assert_eq!(q.len(), 384);
        assert!(
            cosine(&q, &related) > cosine(&q, &unrelated),
            "related text should rank above unrelated"
        );
    }
}
