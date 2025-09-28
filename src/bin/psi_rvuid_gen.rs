use clap::Parser;
use log::debug;
use std::path::Path;

use psitool::logger;
use psitool::rvuid::Rvuid;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, help = "verbose logging (debug logs)")]
    verbose: bool,

    #[arg(short, long, help = "quiet logging (warn+ logs)")]
    quiet: bool,

    #[arg(help = "paths to hash and determine rvuid")]
    paths: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    logger::init(args.verbose, args.quiet);
    for path in args.paths {
        let path = Path::new(&path);
        if path.is_dir() {
            debug!("Skipping {} - it's a directory.", path.display());
            continue;
        }
        let rvuid = Rvuid::from_path(path)?;
        println!("{} = {}", path.display(), rvuid);
    }
    Ok(())
}
