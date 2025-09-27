use log::{error, info};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    target_pools: HashMap<String, TargetPool>,
}

#[derive(Debug, Deserialize)]
pub struct TargetPool {
    pub path: String,
    pub labels: Vec<String>,
    pub wiki: Option<WikiConfig>,
}

#[derive(Debug, Deserialize)]
pub struct WikiConfig {
    pub default_limit: Option<usize>,
    pub queries: Vec<QueryConfig>,
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

    pub fn iter_queries(&self, pool: &str, default_limit: Option<usize>) -> Vec<(&str, usize)> {
        if let Some(tpool) = self.get_pool(pool) {
            tpool.iter_queries(default_limit)
        } else {
            error!("cant find pool '{}'", pool);
            Vec::new()
        }
    }

    pub fn dest_dir(&self, pool: &str) -> anyhow::Result<PathBuf> {
        if let Some(tpool) = self.get_pool(pool) {
            tpool.dest_dir()
        } else {
            error!("cant find pool '{}'", pool);
            anyhow::bail!("couldnt find pool '{}'", pool);
        }
    }
}

impl TargetPool {
    pub fn iter_queries(&self, default_limit: Option<usize>) -> Vec<(&str, usize)> {
        let mut out = Vec::new();

        if let Some(wiki) = &self.wiki {
            for q in &wiki.queries {
                let limit = q
                    .limit
                    .or(default_limit)
                    .or(wiki.default_limit)
                    .unwrap_or(100);

                out.push((q.query.as_str(), limit));
            }
        }
        out
    }

    pub fn dest_dir(&self) -> anyhow::Result<PathBuf> {
        let expanded = shellexpand::tilde(&self.path).into_owned();
        let pbuf = PathBuf::from(expanded);
        if !pbuf.exists() {
            info!("creating directory: {}", pbuf.display());
            std::fs::create_dir_all(&pbuf)?;
        }
        Ok(pbuf)
    }
}
