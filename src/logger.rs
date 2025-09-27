use env_logger::{Builder, Target};
use log::LevelFilter;
use std::io::Write;

pub fn init(verbose: bool) {
    let mut builder = Builder::from_default_env();

    builder
        .format_timestamp_secs()
        .target(Target::Stdout)
        .filter_level(if verbose {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .format(|buf, record| {
            let ts = buf.timestamp();
            writeln!(buf, "[{}] {}: {}", ts, record.level(), record.args())
        });

    builder.init();
}
