#![allow(dead_code)]
use log::error;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    target_pools: HashMap<String, TargetPool>,
}

#[derive(Debug, Deserialize)]
pub struct TargetPool {
    path: String,
    labels: Vec<String>,
    wiki: Option<WikiConfig>,
    wiki_default_limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct WikiConfig {
    default_limit: Option<usize>,
    queries: Vec<QueryConfig>,
}

#[derive(Debug, Deserialize)]
pub struct QueryConfig {
    pub query: String,
    pub limit: Option<usize>,
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let expanded = shellexpand::tilde(path).into_owned();
        let pbuf = PathBuf::from(&expanded);
        if !pbuf.exists() {
            error!("config file '{}' doesnt exist!", expanded);
            anyhow::bail!("config file '{}' doesnt exist", expanded);
        }
        let text = std::fs::read_to_string(pbuf)?;
        let cfg: Config = serde_yaml::from_str(&text)?;
        Ok(cfg)
    }

    pub fn has_pool(&self, pool: &str) -> bool {
        self.target_pools.contains_key(pool)
    }

    pub fn get_pool(&self, pool: &str) -> Option<&TargetPool> {
        self.target_pools.get(pool)
    }

    pub fn list_pools(&self) -> Vec<String> {
        self.target_pools.keys().cloned().collect()
    }
}
