use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatSong {
    pub whitelist: HashSet<String>,
    pub address: String,
}
