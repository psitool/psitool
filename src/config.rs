use anyhow::Context;
use log::{error, info};
use rand::distr::weighted::WeightedIndex;
use rand::prelude::IndexedRandom;
use rand::prelude::*;
use rand::rng;
use serde::Deserialize;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

use crate::rvuid::Rvuid;

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
            fs::create_dir_all(&pbuf)?;
        }
        Ok(pbuf)
    }

    pub fn random_target(&self) -> anyhow::Result<(PathBuf, Option<PathBuf>, Rvuid)> {
        let dir = self.dest_dir()?;
        let jpgs: Vec<PathBuf> = fs::read_dir(dir.clone())?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let p = e.path();
                    match p
                        .extension()
                        .and_then(OsStr::to_str)
                        .map(|s| s.to_ascii_lowercase())
                    {
                        Some(ext) if ext == "jpg" || ext == "jpeg" => Some(p),
                        _ => None,
                    }
                })
            })
            .collect();

        if jpgs.is_empty() {
            anyhow::bail!("no JPG/JPEG files found in {}", dir.display());
        }

        let mut rng = rng();
        let chosen = jpgs
            .choose(&mut rng)
            .cloned()
            .expect("should have chosen a jpeg");

        let yaml_match = chosen.file_stem().and_then(OsStr::to_str).and_then(|stem| {
            let candidate = dir.join(format!("{}.yaml", stem));
            if candidate.exists() {
                Some(candidate)
            } else {
                None
            }
        });

        let img_bytes = fs::read(&chosen)
            .with_context(|| format!("failed to read image bytes from {}", chosen.display()))?;
        let rvuid = Rvuid::from_bytes(&img_bytes);

        Ok((chosen, yaml_match, rvuid))
    }

    pub fn total_images(&self) -> anyhow::Result<usize> {
        let dir = self.dest_dir()?;
        let count = fs::read_dir(dir)?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let p = e.path();
                    match p
                        .extension()
                        .and_then(OsStr::to_str)
                        .map(|s| s.to_ascii_lowercase())
                    {
                        Some(ext) if ext == "jpg" || ext == "jpeg" => Some(()),
                        _ => None,
                    }
                })
            })
            .count();
        Ok(count)
    }
}

pub fn random_pool<'a>(tpools: &'a [&TargetPool]) -> anyhow::Result<&'a TargetPool> {
    let mut rng = rand::rng();

    let weights: Vec<usize> = tpools
        .iter()
        .map(|tp| tp.total_images().unwrap_or(0)) // handle errors gracefully
        .collect();

    if weights.iter().all(|&w| w == 0) {
        anyhow::bail!("all pools are empty");
    }

    let dist = WeightedIndex::new(&weights)
        .map_err(|_| anyhow::anyhow!("failed to build WeightedIndex to choose target pool"))?;

    let idx = dist.sample(&mut rng);
    Ok(tpools[idx])
}
