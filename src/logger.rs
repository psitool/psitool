use env_logger::{Builder, Target};
use log::LevelFilter;
use std::io::Write;

// const DEFAULT_LOG_PATH: &str = "~/psitool.log";

pub fn init(verbose: bool, quiet: bool) {
    let mut builder = Builder::from_default_env();

    builder
        .format_timestamp_secs()
        .filter_level(if quiet {
            LevelFilter::Warn
        } else if verbose {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .format(|buf, record| {
            let ts = buf.timestamp();
            writeln!(buf, "[{}] {}: {}", ts, record.level(), record.args())
        })
        .target(Target::Stderr);

    builder.init();
}
