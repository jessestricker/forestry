use clap::Parser;
use clap_verbosity_flag::{Verbosity, WarnLevel};
use log::info;

#[derive(Debug, Parser)]
#[clap(version, author, about, bin_name = env!("CARGO_BIN_NAME"))]
struct Cli {
    #[clap(flatten)]
    verbosity: Verbosity<WarnLevel>,
}

fn main() {
    // parse CLI args
    let cli = Cli::parse();

    // set up logger
    env_logger::Builder::new()
        .filter_level(cli.verbosity.log_level_filter())
        .format_target(false)
        .format_timestamp(None)
        .init();

    info!("Hello, world!")
}
