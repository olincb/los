use std::hash::{Hash, Hasher};
use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum SourceError {
    #[error("network error: {0}")]
    Network(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("data error: {0}")]
    Data(String),
}

#[derive(Debug, Clone, Hash)]
pub enum Location {
    LocalPath(PathBuf),
    RemoteUrl(String),
}

impl Location {
    pub fn cache_hash(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}
