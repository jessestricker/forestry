use std::fmt::Write as _;
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{anyhow, Context};
use clap::{ArgAction, Args, Parser, ValueEnum};
use log::{error, trace};

use forestry::project::Project;

/// 🌳 Keep your project directory trees in shape!
#[derive(Parser, Debug)]
#[clap(version, author)]
struct Cli {
    root_dir: Option<PathBuf>,

    #[clap(flatten)]
    logger_config: LoggerConfig,
}

#[derive(Args, Debug)]
struct LoggerConfig {
    /// Increase the logging level with each occurrence.
    #[clap(action=ArgAction::Count, short = 'v', long = "verbose")]
    verbose: u8,

    /// Decrease the logging level with each occurrence.
    #[clap(action=ArgAction::Count, short = 'q', long = "quiet")]
    quiet: u8,

    /// Set whether the terminal output includes color.
    #[clap(long = "color", value_enum, default_value_t)]
    color: ColorMode,
}

#[derive(ValueEnum, Clone, Debug, Default)]
enum ColorMode {
    #[default]
    Auto,
    Always,
    Never,
}

impl LoggerConfig {
    const DEFAULT_LEVEL_FILTER: log::LevelFilter = log::LevelFilter::Info;

    pub fn level_filter(&self) -> log::LevelFilter {
        let default_index = log::LevelFilter::iter()
            .enumerate()
            .find(|x| x.1 == Self::DEFAULT_LEVEL_FILTER)
            .expect("default log level should be in list off all log levels")
            .0;
        let index = default_index as i16 + self.verbose as i16 - self.quiet as i16;
        log::LevelFilter::iter()
            .nth(index.max(0) as usize)
            .unwrap_or_else(log::LevelFilter::max)
    }

    pub fn write_style(&self) -> env_logger::WriteStyle {
        use env_logger::WriteStyle;
        match self.color {
            ColorMode::Auto => WriteStyle::Auto,
            ColorMode::Always => WriteStyle::Always,
            ColorMode::Never => WriteStyle::Never,
        }
    }
}

fn setup_cli() -> Cli {
    let args: Cli = Cli::parse();
    env_logger::Builder::new()
        .filter_level(args.logger_config.level_filter())
        .format_target(false)
        .format_timestamp(None)
        .write_style(args.logger_config.write_style())
        .init();
    args
}

fn try_main() -> anyhow::Result<()> {
    let cli = setup_cli();
    trace!("cli = {:#?}", &cli);

    let project = Project::load(cli.root_dir).context("failed to load project")?;
    trace!("project = {:#?}", &project);

    if !project.run() {
        return Err(anyhow!("something failed"));
    }

    Ok(())
}

fn main() -> ExitCode {
    if let Err(err) = try_main() {
        let mut err_msg = err.to_string();
        err.chain().skip(1).for_each(|source| {
            write!(err_msg, "\ncause: {}", source).unwrap();
        });

        error!("{}", err_msg);
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
