use serde::{Deserialize, Serialize};
use std::path::Path;

mod directories;
pub use directories::*;

pub mod secrets;
pub use secrets::Secrets;

mod whatsong;
pub use whatsong::WhatSong;

mod shakespeare;
pub use shakespeare::Shakespeare;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub user_name: String,
    pub owners: Vec<String>,
    pub rooms: Vec<String>,
    pub shakespeare: Shakespeare,
    pub whatsong: WhatSong,
}

impl Config {
    pub fn write_default(path: &Path) -> anyhow::Result<()> {
        let default = Self {
            user_name: "shaken_bot".into(),
            owners: vec!["museun".into()],
            rooms: vec!["#museun".into()],
            shakespeare: Default::default(),
            whatsong: Default::default(),
        };
        std::fs::write(path, toml::to_string_pretty(&default)?)?;
        Ok(())
    }

    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let s = std::fs::read_to_string(path)?;
        let this = toml::from_str(&s)?;
        Ok(this)
    }
}
