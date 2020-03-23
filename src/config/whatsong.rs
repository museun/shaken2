use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatSong {
    pub whitelist: HashSet<String>,
    pub address: String,
}

impl Default for WhatSong {
    fn default() -> Self {
        Self {
            whitelist: vec!["museun"]
                .into_iter()
                .map(ToString::to_string)
                .collect(),
            address: "http://localhost:58810".into(),
        }
    }
}
