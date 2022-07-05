mod project;

use std::process::ExitCode;

use anyhow::Error;
use clap::Parser;
use clap_verbosity_flag::{Verbosity, WarnLevel};
use log::{error, info, warn};

#[derive(Debug, Parser)]
#[clap(version, author, about, bin_name = env!("CARGO_BIN_NAME"))]
struct Cli {
    #[clap(flatten)]
    verbosity: Verbosity<WarnLevel>,
}

fn main() -> ExitCode {
    // parse CLI args
    let cli = Cli::parse();

    // set up logger
    env_logger::Builder::new()
        .filter_level(cli.verbosity.log_level_filter())
        .format_target(false)
        .format_timestamp(None)
        .init();

    // run application
    let result = run();
    if let Err(err) = result {
        error!("{:?}", err);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn run() -> Result<(), Error> {
    // load project
    let project = if let Some(x) = project::Project::load_nearest()? {
        x
    } else {
        warn!("no project configuration file found");
        return Ok(());
    };

    info!("project = {:#?}", &project);

    Ok(())
}
