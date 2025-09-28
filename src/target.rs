use std::collections::HashMap;
//use serde::Deserialize;

#[derive(Debug, serde::Serialize)]
pub struct YamlData {
    pub query: String,
    pub frontloading: Vec<String>,
    pub image_description: serde_json::Value,
    pub datetime_original: serde_json::Value,
    pub img_metadata: HashMap<String, serde_json::Value>,
    pub license: String,
    pub license_meta: HashMap<String, serde_json::Value>,
}

pub enum TargetType {
    Text,
    ImageType,
}

pub struct Target {
    pub target_type: TargetType,
    pub path: String,
    pub meta_path: String,
    pub meta: HashMap<String, serde_json::Value>,
    pub frontloading: Vec<String>,
}
