//! Runtime configuration and on-disk layout.

use std::path::{Path, PathBuf};

/// Where Cairn keeps its data. Defaults to the OS data dir (`~/.local/share/cairn`,
/// `%APPDATA%\cairn`, …) but can be overridden (e.g. `/data` in Docker).
#[derive(Debug, Clone)]
pub struct Config {
    pub data_dir: PathBuf,
}

impl Config {
    /// Resolve the data dir (creating it if needed). `None` uses the OS default.
    pub fn resolve(data_dir: Option<PathBuf>) -> crate::Result<Self> {
        let data_dir = match data_dir {
            Some(d) => d,
            None => default_data_dir(),
        };
        std::fs::create_dir_all(&data_dir)?;
        let cfg = Self { data_dir };
        std::fs::create_dir_all(cfg.blobs_dir())?;
        Ok(cfg)
    }

    pub fn db_path(&self) -> PathBuf {
        self.data_dir.join("cairn.db")
    }

    pub fn blobs_dir(&self) -> PathBuf {
        self.data_dir.join("blobs")
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }
}

fn default_data_dir() -> PathBuf {
    if let Some(dirs) = directories::ProjectDirs::from("dev", "cairn", "cairn") {
        dirs.data_dir().to_path_buf()
    } else {
        PathBuf::from(".cairn")
    }
}
