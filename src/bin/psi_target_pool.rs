use clap::Parser;
use log::{info, warn};
use std::io::{self, Write};

use psitool::config::{Config, TargetPool, random_pool};
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
    logger::init(args.verbose);
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
        let tpool_total = tpool.total_images()?;
        total += tpool_total;
        info!("pool {}: {} jpgs", tpool.path, tpool_total);
    }
    info!("Total jpgs: {}", total);

    let tpool = random_pool(&tpools)?;
    let (img_path, _yaml_path, rvuid) = tpool.random_target()?;
    info!("Chose rvuid {}", rvuid);
    println!("Target: {}", rvuid);
    println!("Press ENTER to see target.");
    println!("Remote viewer, begin.");
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    open::that(&img_path)?;
    Ok(())
}
