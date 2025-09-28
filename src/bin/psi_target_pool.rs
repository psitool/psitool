use clap::{ArgAction, Parser};
use log::{info, warn};
use std::io::{self, Write};

use psitool::config::{Config, TargetPool, random_pool};
use psitool::logger;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, help = "verbose logging (debug logs)")]
    verbose: bool,

    #[arg(short, long, help = "quiet logging (warn+ logs)")]
    quiet: bool,

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
            info!(
                "excluding '{}' pool due to exclude label '{}'",
                pool, exclude
            );
        } else if args.pools.contains(&pool) {
            info!("including pool '{}' by name", pool);
            tpools.push(tpool);
        } else if let Some(ref include) = args.include_label
            && tpool.labels.contains(include)
        {
            info!("including pool '{}' by label {}", pool, include);
            tpools.push(tpool);
        } else if args.pools.is_empty()
            && args.include_label.is_none()
            && args.exclude_label.is_none()
        {
            info!(
                "including pool '{}' because no options passed (all pools)",
                pool
            );
            tpools.push(tpool);
        }
    }
    info!("found {} target pools to match", tpools.len());
    let mut total = 0usize;
    for tpool in tpools.clone() {
        let tpool_total = tpool.total_targets()?;
        total += tpool_total;
        info!("pool {}: {} targets", tpool.path, tpool_total);
    }
    info!("Total targets: {}", total);

    let tpool = random_pool(&tpools)?;
    let target = tpool.random_target()?;
    info!("Chose rvuid {}", target.rvuid);
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
    if !args.skip_open {
        open::that(&target.path)?;
    }
    println!("Path: {}", target.path.display());
    if let Some(ref meta_path) = target.meta_path {
        println!("YAML meta: {}", meta_path.display());
    }
    if !target.meta.is_empty() {
        for (key, val) in target.iter_meta() {
            println!("{}: {}", key, val);
        }
    }
    Ok(())
}
