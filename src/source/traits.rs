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

#[derive(Debug, Clone)]
pub enum Location {
    LocalPath(PathBuf),
    RemoteUrl(String),
}
