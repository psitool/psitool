use clap::Parser;
use log::{debug, info, warn};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::Path;

use psitool::config::Config;
use psitool::logger;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, help = "verbose logging (debug logs)")]
    verbose: bool,

    #[arg(
        short,
        long,
        default_value = "~/.psitool.yaml",
        help = "the config with the target pools"
    )]
    config: String,

    #[arg(short, long, help = "override wiki default_limit")]
    limit: Option<usize>,

    #[arg(help = "the target pool to download for")]
    pool: String,
}

static RE_CC_BY: Lazy<Regex> = Lazy::new(|| Regex::new(r"^CC BY(\s+\d+(\.\d+)?)?$").unwrap());

fn make_user_agent() -> String {
    format!(
        "{}/psi-wm-downloader/{} ({})",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS")
    )
}

const API: &str = "https://commons.wikimedia.org/w/api.php";

#[derive(Debug, Deserialize)]
struct ApiResponse {
    query: Option<Query>,
}

#[derive(Debug, Deserialize)]
struct Query {
    pages: HashMap<String, Page>,
}

#[derive(Debug, Deserialize)]
struct Page {
    title: String,
    imageinfo: Option<Vec<ImageInfo>>,
}

impl fmt::Display for Page {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Page({})", self.title)
    }
}

#[derive(Debug, Deserialize)]
struct ImageInfo {
    url: String,
    extmetadata: Option<HashMap<String, ExtValue>>,
}

#[derive(Debug, Deserialize)]
struct ExtValue {
    value: serde_json::Value,
}

#[derive(Debug, serde::Serialize)]
struct YamlData {
    query: String,
    image_description: serde_json::Value,
    datetime_original: serde_json::Value,
    img_metadata: HashMap<String, serde_json::Value>,
    license: String,
    license_meta: HashMap<String, serde_json::Value>,
}

fn make_client() -> anyhow::Result<reqwest::blocking::Client> {
    let user_agent = make_user_agent();
    debug!("using user agent: {}", user_agent);
    let client = reqwest::blocking::Client::builder()
        .user_agent(user_agent)
        .build()?;
    Ok(client)
}

fn search_images(query: &str, limit: usize) -> anyhow::Result<Vec<Page>> {
    debug!("searching for query {} with limit {}", query, limit);
    let client = make_client()?;
    let params = [
        ("action", "query"),
        ("generator", "search"),
        ("gsrsearch", query),
        ("gsrlimit", &limit.to_string()),
        ("gsrnamespace", "6"), // only File: pages
        ("prop", "imageinfo"),
        ("iiprop", "url|extmetadata"),
        ("format", "json"),
    ];
    let resp = client.get(API).query(&params).send()?;
    if !resp.status().is_success() {
        anyhow::bail!("API error: {}", resp.status());
    }
    let resp: ApiResponse = resp.json()?;
    let pages = resp
        .query
        .map(|q| q.pages.into_values().collect())
        .unwrap_or_default();
    Ok(pages)
}

fn valid_license(license: &str) -> bool {
    let normalized = license.replace('-', " ").to_uppercase();
    debug!("Checking license string: '{}'", normalized);
    normalized == "CC0" || normalized == "PUBLIC DOMAIN" || RE_CC_BY.is_match(&normalized)
}

fn download_and_save(
    query: &str,
    page: &Page,
    out_dir: &str,
) -> anyhow::Result<Option<(String, String)>> {
    debug!("download_and_save {} to {}", page, out_dir);
    let client = make_client()?;
    let Some(info) = page.imageinfo.as_ref().and_then(|v| v.first()) else {
        debug!("Couldnt get page.imageinfo of {}", page);
        return Ok(None);
    };

    let blank = HashMap::new();
    let meta = info.extmetadata.as_ref().unwrap_or(&blank);

    let license_short = meta
        .get("LicenseShortName")
        .map(|v| v.value.to_string())
        .unwrap_or("".to_string())
        .replace('"', "");

    if !valid_license(&license_short) {
        info!("INVALID license: {} was {}", page, license_short);
        return Ok(None);
    } else {
        info!("validated license: {} was {}", page, license_short);
    }

    fs::create_dir_all(out_dir)?;

    let title = page.title.trim_start_matches("File:").replace(' ', "_");
    let filename = format!("{}/{}", out_dir, title);

    if !Path::new(&filename).exists() {
        info!("Downloading {} from {}: {}", page, info.url, filename);
        let bytes = client.get(&info.url).send()?.bytes()?;
        let mut file = fs::File::create(&filename)?;
        file.write_all(&bytes)?;
    }

    let image_description = meta
        .get("ImageDescription")
        .map(|v| v.value.clone())
        .unwrap_or_default();
    let datetime_original = meta
        .get("DateTimeOriginal")
        .map(|v| v.value.clone())
        .unwrap_or_default();

    let mut img_metadata = HashMap::new();
    let mut license_meta = HashMap::new();

    for (k, v) in meta {
        match k.as_str() {
            "License"
            | "LicenseUrl"
            | "LicenseShortName"
            | "UsageTerms"
            | "AttributionRequired"
            | "Artist"
            | "Permission"
            | "Restrictions"
            | "Copyrighted"
            | "Credit" => {
                license_meta.insert(k.clone(), v.value.clone());
            }
            _ => {
                img_metadata.insert(k.clone(), v.value.clone());
            }
        }
    }

    let yaml_data = YamlData {
        query: query.to_string(),
        image_description: image_description.clone(),
        datetime_original: datetime_original.clone(),
        img_metadata,
        license: license_short,
        license_meta,
    };

    let yaml_path = format!("{}.yaml", filename);
    let mut f = fs::File::create(&yaml_path)?;
    serde_yaml::to_writer(&mut f, &yaml_data)?;

    Ok(Some((filename, yaml_path)))
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    logger::init(args.verbose);
    let cfg = Config::load(&args.config)?;
    info!("pool chosen was: {}", args.pool);
    if !cfg.has_pool(&args.pool) {
        warn!(
            "cant find pool '{}' of pools: {:?}",
            args.pool,
            cfg.list_pools()
        );
        anyhow::bail!("pool '{}' not found!'", args.pool);
    }
    let tpool = cfg.get_pool(&args.pool).unwrap();
    for (query, limit) in tpool.iter_queries(args.limit) {
        info!("query {} and limit {}", query, limit);
        let dest_dir_buf = tpool.dest_dir()?;
        let dest_dir = dest_dir_buf.to_str().unwrap();
        let results = search_images(query, limit)?;
        for page in results {
            if let Some((img, meta)) = download_and_save(query, &page, dest_dir)? {
                info!("Saved img {} and metadata {}", img, meta);
            } else {
                warn!("didnt get anything with Page {}", page);
            }
        }
    }
    Ok(())
}
