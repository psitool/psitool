use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::rvuid::Rvuid;
use crate::target::Target;

#[derive(Default, Clone, Debug)]
pub struct CacheMap(HashMap<PathBuf, CachedHash>);

impl CacheMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Runs canonicalize on the path and always expects it to work.
    pub fn insert(&mut self, key: PathBuf, value: CachedHash) {
        let canon = std::fs::canonicalize(&key).unwrap();
        self.0.insert(canon, value);
    }

    /// Runs canonicalize on the path and always expects it to work.
    pub fn get(&mut self, key: &PathBuf) -> Option<&CachedHash> {
        let canon = std::fs::canonicalize(key).unwrap();
        self.0.get(&canon)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl IntoIterator for CacheMap {
    type Item = (PathBuf, CachedHash);
    type IntoIter = std::collections::hash_map::IntoIter<PathBuf, CachedHash>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<(PathBuf, CachedHash)> for CacheMap {
    fn from_iter<I: IntoIterator<Item = (PathBuf, CachedHash)>>(iter: I) -> Self {
        CacheMap(iter.into_iter().collect::<HashMap<PathBuf, CachedHash>>())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CachedHash {
    pub rvuid: Rvuid,
    pub path: PathBuf,
}

impl fmt::Display for CachedHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CachedHash[{}]", self.rvuid)
    }
}

impl TryFrom<CachedHash> for Target {
    type Error = anyhow::Error;

    fn try_from(ch: CachedHash) -> Result<Self, Self::Error> {
        Target::parse(&ch.path)
    }
}

impl From<Target> for CachedHash {
    fn from(target: Target) -> Self {
        Self {
            rvuid: target.rvuid.clone(),
            path: target.path,
        }
    }
}

impl CachedHash {
    /// Given a path to a cache config, parse all cached hashes.
    pub fn parse(path: &str) -> anyhow::Result<CacheMap> {
        let expanded = shellexpand::tilde(path).into_owned();
        let pbuf = PathBuf::from(&expanded);
        if !pbuf.exists() {
            warn!("{} doesnt exist, not loading cached hashes", pbuf.display());
            let empty: CacheMap = CacheMap::new();
            return Ok(empty);
        }
        debug!("Parsing cached hashes at {}", pbuf.display());
        let text = std::fs::read_to_string(pbuf)?;
        let cache: Vec<CachedHash> = serde_yaml::from_str(&text)?;
        debug!("Found {} cached hashes", cache.len());
        let cachemap: CacheMap = cache.into_iter().map(|ch| (ch.path.clone(), ch)).collect();
        Ok(cachemap)
    }

    /// Save the cached hashes back to a file.
    pub fn dump(cachemap: &CacheMap, path: &str) -> anyhow::Result<()> {
        let cached_hashes: Vec<Self> = cachemap
            .clone()
            .into_iter()
            .map(|(_, ch)| ch.clone())
            .collect();
        let expanded = shellexpand::tilde(path).into_owned();
        let pbuf = PathBuf::from(&expanded);
        debug!(
            "Writing {} cached hashes to {}",
            cached_hashes.len(),
            pbuf.display()
        );
        let yaml = serde_yaml::to_string(&cached_hashes)?;
        let mut file = File::create(pbuf.clone())?;
        file.write_all(yaml.as_bytes())?;
        debug!(
            "Successfully wrote {} cached hashes to {}",
            cached_hashes.len(),
            pbuf.display()
        );
        Ok(())
    }
}
