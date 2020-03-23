use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shakespeare {
    pub whitelist: HashSet<String>,
    pub address: String,
    pub chance: f32,
    pub quiet: u64,
    pub interval: u64,
}

impl Default for Shakespeare {
    fn default() -> Self {
        Self {
            whitelist: vec!["museun"]
                .into_iter()
                .map(ToString::to_string)
                .collect(),
            address: "http://localhost:9090".into(),
            chance: 0.5,
            quiet: 300,
            interval: 30,
        }
    }
}
