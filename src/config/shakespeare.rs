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
