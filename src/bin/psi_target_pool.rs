use clap::{ArgAction, Parser};
use log::{debug, error, info, warn};
use std::io::{self, Write};

use psitool::cache::{CacheMap, CachedHash};
use psitool::config::{Config, TargetPool, random_pool};
use psitool::logger;
use psitool::rvuid::Rvuid;
use psitool::target::{CompletedTarget, TargetType};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, help = "verbose logging (debug logs)")]
    verbose: bool,

    #[arg(short, long, help = "quiet logging (warn+ logs)")]
    quiet: bool,

    #[arg(
        short,
        long,
        help = "reuse all targets, even if they're already completed"
    )]
    reuse_targets: bool,

    #[arg(short, action = ArgAction::Count, help = "how much to frontload, none by default (pass -f for 1 level of frontloading, -ff for 2, -fff for 3...)")]
    frontload: u8,

    #[arg(short, long, help = "dont open the target after")]
    skip_open: bool,

    #[arg(
        short,
        long,
        default_value = "~/.psitool.yaml",
        help = "the config with the target pools"
    )]
    config: String,

    #[arg(
        short = 'C',
        long,
        default_value = "~/.psitool_completed_targets.yaml",
        help = "the yaml config with a list of completed targets (used to cache what you RV'd already)"
    )]
    completed: String,

    #[arg(
        long,
        default_value = "~/.psitool_cached_hashes.yaml",
        help = "the yaml config with a list of cached hashes so it doesn't have to compute them every run"
    )]
    cached_hashes: String,

    #[arg(
        short,
        long,
        help = "the named target pool to read from (included unless excluded via label)"
    )]
    pools: Vec<String>,

    #[arg(
        short = 'i',
        long,
        help = "the target pools to read from, including this label"
    )]
    include_label: Option<String>,

    #[arg(
        short = 'x',
        long,
        help = "the target pools to read from, EXCLUDING this label"
    )]
    exclude_label: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    logger::init(args.verbose, args.quiet);
    let cfg = Config::load(&args.config)?;
    let mut completed_targets: Vec<CompletedTarget> = CompletedTarget::parse(&args.completed)?;
    let mut cachemap: CacheMap = CachedHash::parse(&args.cached_hashes)?;
    let mut tpools: Vec<&TargetPool> = Vec::new();
    for pool in args.pools.clone() {
        if !cfg.has_pool(&pool) {
            warn!("cant find passed pool '{}'", pool);
            anyhow::bail!("couldnt find pool '{}'", pool);
        }
    }
    for pool in cfg.list_pools() {
        let tpool = cfg.get_pool(&pool).unwrap();
        if let Some(ref exclude) = args.exclude_label
            && tpool.labels.contains(exclude)
        {
            debug!(
                "excluding '{}' pool due to exclude label '{}'",
                pool, exclude
            );
        } else if args.pools.contains(&pool) {
            debug!("including pool '{}' by name", pool);
            tpools.push(tpool);
        } else if let Some(ref include) = args.include_label
            && tpool.labels.contains(include)
        {
            debug!("including pool '{}' by label {}", pool, include);
            tpools.push(tpool);
        } else if args.pools.is_empty()
            && args.include_label.is_none()
            && args.exclude_label.is_none()
        {
            debug!(
                "including pool '{}' because no options passed (all pools)",
                pool
            );
            tpools.push(tpool);
        }
    }
    let completed_rvuids: Vec<Rvuid> = if args.reuse_targets {
        Vec::new()
    } else {
        completed_targets.iter().map(|t| t.rvuid.clone()).collect()
    };
    debug!("found {} target pools to pull target from", tpools.len());
    let mut total = 0usize;
    for tpool in tpools.clone() {
        let tpool_total = tpool.total_targets(&completed_rvuids, &mut cachemap)?;
        total += tpool_total;
        debug!("pool {}: {} targets", tpool.path, tpool_total);
    }
    info!("Selecting from {} pools, {} targets.", tpools.len(), total);

    let tpool = random_pool(&tpools, &completed_rvuids, &mut cachemap)?;
    let target = tpool.random_target(&completed_rvuids, &mut cachemap)?;
    debug!("Chose rvuid {}", target.rvuid);
    println!("Target: {}", target.rvuid);
    if args.frontload > 0 {
        let range_end: usize = target.frontloading.len().min(args.frontload as usize);
        let frontloading = &target.frontloading[..range_end];
        println!("Frontloading: {:?}", frontloading);
    }
    println!("Remote viewer, begin viewing.");
    println!("Press ENTER when complete.");
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    println!("Path: {}", target.path.display());
    if target.target_type == TargetType::Text {
        match std::fs::read_to_string(&target.path) {
            Ok(contents) => println!("Target Text:\n{}", contents),
            _ => {
                if args.skip_open {
                    error!("Failed to read target text. You may need to open it manually.");
                } else {
                    open::that(&target.path)?;
                }
            }
        }
    } else if !args.skip_open {
        // We don't open text files if we can read them above as text.
        open::that(&target.path)?;
    }
    if let Some(ref meta_path) = target.meta_path {
        println!("YAML meta: {}", meta_path.display());
    }
    if !target.meta.is_empty() {
        for (key, val) in target.iter_meta() {
            println!("{}: {}", key, val);
        }
    }
    let mut completed_target = CompletedTarget::from(target);
    completed_target.interactive_ask_results();
    info!("Adding completed target {}", completed_target);
    completed_targets.push(completed_target);
    CompletedTarget::dump(&completed_targets, &args.completed)?;
    CachedHash::dump(&cachemap, &args.cached_hashes)?;
    Ok(())
}
