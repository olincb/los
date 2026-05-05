use std::path::PathBuf;
use std::sync::OnceLock;

pub fn cache_directory() -> &'static Option<PathBuf> {
    static CACHE_DIR: OnceLock<Option<PathBuf>> = OnceLock::new();
    CACHE_DIR.get_or_init(|| {
        home::home_dir()
            .filter(|path| !path.as_os_str().is_empty())
            .map(|path| path.join(".cache").join("los"))
    })
}
