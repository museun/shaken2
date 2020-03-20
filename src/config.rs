use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub user_name: String,
    pub rooms: Vec<String>,
    pub shakespeare: Shakespeare,
    pub whatsong: WhatSong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shakespeare {
    pub whitelist: HashSet<String>,
    pub address: String,
    pub chance: f32,
    pub quiet: u64,
    pub interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatSong {
    pub whitelist: HashSet<String>,
    pub address: String,
}

impl Config {
    pub fn write_default(path: &Path) -> anyhow::Result<()> {
        let default = Self {
            user_name: "shaken_bot".into(),
            rooms: vec!["#museun".into()],
            shakespeare: Shakespeare {
                whitelist: vec!["museun"]
                    .into_iter()
                    .map(ToString::to_string)
                    .collect(),
                address: "http://localhost:9090".into(),
                chance: 0.5,
                quiet: 300,
                interval: 30,
            },
            whatsong: WhatSong {
                whitelist: vec!["museun"]
                    .into_iter()
                    .map(ToString::to_string)
                    .collect(),
                address: "http://localhost:58810".into(),
            },
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

fn parse_and_set(data: &str, set: fn(k: &str, v: &str)) {
    data.lines()
        .filter_map(|s| Some(s.trim()).filter(|s| !s.starts_with('#')))
        .map(|s| {
            s.splitn(2, '=')
                .filter_map(|s| Some(s.trim()).filter(|s| !s.is_empty()))
        })
        .flat_map(|mut iter| Some((iter.next()?, iter.next()?)))
        .for_each(|(k, v)| set(k, &v.replace('"', "")))
}

pub fn load_env_from<I: IntoIterator<Item = T>, T: AsRef<std::path::Path>>(paths: I) {
    for data in paths
        .into_iter()
        .flat_map(|path| std::fs::read_to_string(path).ok())
    {
        parse_and_set(&data, |k, v| std::env::set_var(&k, &v))
    }
}
