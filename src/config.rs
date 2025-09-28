use log::{error, info};
use rand::distr::weighted::WeightedIndex;
use rand::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::target::Target;

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
    query: String,
    limit: Option<usize>,
    frontloading: Option<Vec<String>>,
}

pub struct Query {
    pub query: String,
    pub limit: usize,
    pub frontloading: Vec<String>,
}

impl std::fmt::Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Query({}, {}, {:?})",
            self.query, self.limit, self.frontloading
        )
    }
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let expanded = shellexpand::tilde(path).into_owned();
        let pbuf = PathBuf::from(&expanded);
        if !pbuf.exists() {
            error!("config file '{}' doesnt exist!", expanded);
            anyhow::bail!("config file '{}' doesnt exist", expanded);
        }
        let text = fs::read_to_string(pbuf)?;
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

    pub fn iter_queries(&self, pool: &str, default_limit: Option<usize>) -> Vec<Query> {
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
    pub fn iter_queries(&self, default_limit: Option<usize>) -> Vec<Query> {
        let mut out = Vec::new();

        if let Some(wiki) = &self.wiki {
            for q in &wiki.queries {
                let limit = q
                    .limit
                    .or(default_limit)
                    .or(wiki.default_limit)
                    .unwrap_or(100);

                let frontloading = q.frontloading.clone().unwrap_or(Vec::new());

                out.push(Query {
                    query: q.query.clone(),
                    limit,
                    frontloading,
                });
            }
        }
        out
    }

    pub fn dest_dir(&self) -> anyhow::Result<PathBuf> {
        let expanded = shellexpand::tilde(&self.path).into_owned();
        let pbuf = PathBuf::from(expanded);
        if !pbuf.exists() {
            info!("creating directory: {}", pbuf.display());
            fs::create_dir_all(&pbuf)?;
        }
        Ok(pbuf)
    }

    pub fn random_target(&self) -> anyhow::Result<Target> {
        let dir = self.dest_dir()?;
        Target::random_from_dir(&dir)
    }

    pub fn total_targets(&self) -> anyhow::Result<usize> {
        let dir = self.dest_dir()?;
        Ok(Target::all_from_dir(&dir)?.len())
    }
}

pub fn random_pool<'a>(tpools: &'a [&TargetPool]) -> anyhow::Result<&'a TargetPool> {
    let mut rng = rand::rng();

    let weights: Vec<usize> = tpools
        .iter()
        .map(|tp| tp.total_targets().unwrap_or(0)) // handle errors gracefully
        .collect();

    if weights.iter().all(|&w| w == 0) {
        anyhow::bail!("all pools are empty");
    }

    let dist = WeightedIndex::new(&weights)
        .map_err(|_| anyhow::anyhow!("failed to build WeightedIndex to choose target pool"))?;

    let idx = dist.sample(&mut rng);
    Ok(tpools[idx])
}
