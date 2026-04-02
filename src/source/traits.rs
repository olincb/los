use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Location {
    LocalPath(PathBuf),
    RemoteUrl(String),
}
