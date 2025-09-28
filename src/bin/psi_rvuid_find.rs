use clap::Parser;
use std::collections::HashSet;

use psitool::config::Config;
use psitool::logger;
use psitool::rvuid::Rvuid;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, help = "verbose logging (debug logs)")]
    verbose: bool,

    #[arg(short, long, help = "quiet logging (warn+ logs)")]
    quiet: bool,

    #[arg(
        short = 'D',
        long,
        help = "keep searching even if you already found every RVUID (find potential dupes)"
    )]
    find_dupes: bool,

    #[arg(
        short,
        long,
        default_value = "~/.psitool.yaml",
        help = "the config with the target pools (this is where it will look for the RVUID)"
    )]
    config: String,

    #[arg(help = "the RVUIDs to look for")]
    rvuids: Vec<Rvuid>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.rvuids.is_empty() {
        return Ok(());
    }
    logger::init(args.verbose, args.quiet);
    let cfg = Config::load(&args.config)?;
    let mut found: HashSet<Rvuid> = HashSet::new();
    let mut missing: HashSet<Rvuid> = args.rvuids.clone().into_iter().collect();
    let orig: HashSet<Rvuid> = args.rvuids.into_iter().collect();
    for pool in cfg.list_pools() {
        let tpool = cfg.get_pool(&pool).unwrap();
        for target in tpool.all_targets()? {
            if orig.contains(&target.rvuid) {
                println!("{} found at: {}", target.rvuid, target.path.display());
                found.insert(target.rvuid.clone());
                missing.remove(&target.rvuid);
                if missing.is_empty() && !args.find_dupes {
                    return Ok(());
                }
            }
        }
    }
    if missing.is_empty() {
        Ok(())
    } else {
        anyhow::bail!("missing: {:?}", missing);
    }
}
