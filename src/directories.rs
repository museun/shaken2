use anyhow::Context as _;
use std::path::PathBuf;

pub struct Directories;
impl Directories {
    const SUBDIR: &'static str = "shaken";

    pub fn config() -> anyhow::Result<PathBuf> {
        Self::get_and_make_dir(dirs::config_dir(), "config")
    }

    pub fn data() -> anyhow::Result<PathBuf> {
        Self::get_and_make_dir(dirs::data_dir(), "data")
    }

    fn get_and_make_dir(dir: Option<PathBuf>, kind: &str) -> anyhow::Result<PathBuf> {
        let path = dir
            .map(|dir| dir.join(Self::SUBDIR))
            .ok_or_else(|| anyhow::anyhow!("cannot get {} directory", kind))?;
        std::fs::create_dir_all(&path)
            .with_context(|| format!("cannot create {} directory: `{}`", kind, path.display()))?;
        Ok(path)
    }
}
