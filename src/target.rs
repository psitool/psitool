use anyhow::Context;
use log::{debug, info, warn};
use rand::prelude::IndexedRandom;
use rand::rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::rvuid::Rvuid;

#[derive(Clone, Debug, serde::Serialize, Deserialize)]
pub struct YamlData {
    pub query: String,
    pub frontloading: Vec<String>,
    pub image_description: serde_json::Value,
    pub datetime_original: serde_json::Value,
    pub img_metadata: HashMap<String, serde_json::Value>,
    pub license: String,
    pub license_meta: HashMap<String, serde_json::Value>,
}

impl YamlData {
    pub fn serialize(&self) -> HashMap<String, String> {
        let mut map: HashMap<String, String> = HashMap::new();
        map.insert("Query".to_string(), self.query.to_string());
        map.insert(
            "Description".to_string(),
            self.image_description.to_string(),
        );
        map.insert("Datetime".to_string(), self.datetime_original.to_string());
        map.insert("License".to_string(), self.license.to_string());
        map
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TargetType {
    Text,
    Jpeg,
    Svg,
}

impl TargetType {
    pub fn parse(path: &Path) -> Option<Self> {
        let ext = path
            .extension()
            .and_then(OsStr::to_str)
            .map(|s| s.to_ascii_lowercase());
        match ext {
            Some(ext) if ext == "jpg" || ext == "jpeg" => Some(TargetType::Jpeg),
            Some(ext) if ext == "svg" => Some(TargetType::Svg),
            Some(ext) if ext == "target" => Some(TargetType::Text),
            Some(ext) if ext == "yaml" || ext == "yml" => None,
            None => {
                warn!("Extension not parsed from: {}", path.display());
                None
            }
            _ => {
                debug!("Ignoring non-target extension: {}", path.display());
                None
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Target {
    pub rvuid: Rvuid,
    pub path: PathBuf,
    pub meta_path: Option<PathBuf>,
    pub frontloading: Vec<String>,
    pub target_type: TargetType,
    pub meta: HashMap<String, String>,
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Target[{}]", self.rvuid)
    }
}

impl Target {
    pub fn parse(path: &Path) -> anyhow::Result<Self> {
        let img_bytes = fs::read(path)
            .with_context(|| format!("failed to read bytes from {}", path.display()))?;
        let rvuid = Rvuid::from_bytes(&img_bytes);
        let maybe_meta_path = Self::maybe_yaml(path);
        let meta_path = maybe_meta_path.clone();
        let target_type = TargetType::parse(path)
            .ok_or_else(|| anyhow::anyhow!("no target at {}", path.display()))?;
        if let Some(unp_meta_path) = maybe_meta_path {
            let text = fs::read_to_string(unp_meta_path)?;
            let yaml_data: YamlData = serde_yaml::from_str(&text)?;
            let frontloading = yaml_data.frontloading.clone();
            Ok(Target {
                rvuid,
                target_type,
                path: path.to_path_buf(),
                meta_path,
                frontloading,
                meta: yaml_data.clone().serialize(),
            })
        } else {
            debug!("No metadata found for file: {:?}", path);
            let frontloading: Vec<String> = Vec::new();
            let blank: HashMap<String, String> = HashMap::new();
            Ok(Target {
                rvuid,
                target_type,
                path: path.to_path_buf(),
                meta_path,
                frontloading,
                meta: blank,
            })
        }
    }

    fn maybe_yaml(path: &Path) -> Option<PathBuf> {
        let new_path = path.with_file_name(format!(
            "{}.yaml",
            path.file_name().unwrap().to_string_lossy()
        ));
        if new_path.exists() {
            Some(new_path)
        } else {
            None
        }
    }
}

impl Target {
    pub fn all_from_dir(dir: &Path, completed_rvuids: &[Rvuid]) -> anyhow::Result<Vec<Target>> {
        let targets: Vec<Target> = fs::read_dir(dir)?
            .filter_map(|entry| entry.ok().and_then(|e| Target::parse(&e.path()).ok()))
            .filter(|t| !completed_rvuids.contains(&t.rvuid))
            .collect();
        Ok(targets)
    }

    pub fn random_from_dir(dir: &Path, completed_rvuids: &[Rvuid]) -> anyhow::Result<Self> {
        let targets = Self::all_from_dir(dir, completed_rvuids)?;
        if targets.is_empty() {
            anyhow::bail!("no JPG/JPEG/SVG or TARGET files found in {}", dir.display());
        }

        let mut rng = rng();
        let chosen = targets
            .choose(&mut rng)
            .cloned()
            .expect("should have chosen a target");

        Ok(chosen)
    }

    pub fn iter_meta(&self) -> Vec<(String, String)> {
        let keys: Vec<String> = vec![
            "Query".into(),
            "Description".into(),
            "Datetime".into(),
            "License".into(),
        ];
        keys.into_iter()
            .map(|key| {
                (
                    key.clone(),
                    self.meta.get(&key).map_or("".to_string(), |v| v.clone()),
                )
            })
            .collect()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CompletedTarget {
    pub rvuid: Rvuid,
    pub path: String,
    pub meta_path: Option<String>,
    pub hit: Option<bool>,
    pub score: Option<u32>,
    pub notes: Option<String>,
}

impl CompletedTarget {
    pub fn parse(path: &str) -> anyhow::Result<Vec<Self>> {
        let expanded = shellexpand::tilde(path).into_owned();
        let pbuf = PathBuf::from(&expanded);
        if !pbuf.exists() {
            let targs: Vec<Self> = Vec::new();
            return Ok(targs);
        }
        let text = fs::read_to_string(pbuf)?;
        let targs: Vec<Self> = serde_yaml::from_str(&text)?;
        Ok(targs)
    }
    pub fn dump(completed_targets: &Vec<Self>, path: &str) -> anyhow::Result<()> {
        let expanded = shellexpand::tilde(path).into_owned();
        let pbuf = PathBuf::from(&expanded);
        debug!(
            "Writing {} completed targets to {} ...",
            completed_targets.len(),
            pbuf.display()
        );
        let yaml = serde_yaml::to_string(completed_targets)?;
        let mut file = File::create(pbuf.clone())?;
        file.write_all(yaml.as_bytes())?;
        info!(
            "Succesfully wrote {} completed targets to {}",
            completed_targets.len(),
            pbuf.display()
        );
        Ok(())
    }
    pub fn interactive_ask_results(&mut self) {
        print!("Was it a hit ([y]es, [n]o, otherwise not saved/recorded)? ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        self.hit = match input.trim().to_lowercase().as_str() {
            "y" | "yes" => Some(true),
            "n" | "no" => Some(false),
            _ => {
                debug!("Not recording whether it's a hit or not.");
                None
            }
        };

        print!("Score out of 100 (0 to 100 or otherwise not saved/recorded)? ");
        io::stdout().flush().unwrap();
        input.clear();
        io::stdin().read_line(&mut input).unwrap();
        self.score = match input.trim().parse::<u32>() {
            Ok(n) if (0..=100).contains(&n) => {
                info!("Recording score of {}", n);
                Some(n)
            }
            _ => {
                debug!("Not recording score.");
                None
            }
        };

        print!("Any notes? Press enter to end (or blank to not save anything): ");
        io::stdout().flush().unwrap();
        input.clear();
        io::stdin().read_line(&mut input).unwrap();
        self.notes = match input.trim() {
            "" => None,
            s => Some(s.to_string()),
        };
    }
}

impl From<Target> for CompletedTarget {
    fn from(target: Target) -> Self {
        CompletedTarget {
            rvuid: target.rvuid,
            path: target.path.display().to_string(),
            meta_path: target.meta_path.map(|p| p.display().to_string()),
            hit: None,
            score: None,
            notes: None,
        }
    }
}

impl fmt::Display for CompletedTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(hit) = self.hit {
            write!(f, "CompletedTarget[{}, {}]", self.rvuid, hit)
        } else {
            write!(f, "CompletedTarget[{}]", self.rvuid)
        }
    }
}
