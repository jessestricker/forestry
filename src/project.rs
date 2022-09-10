use std::env;
use std::path::PathBuf;

use globset::Candidate;
use ignore::overrides::OverrideBuilder;
use ignore::WalkBuilder;
use log::{debug, error, trace, warn};
use thiserror::Error;

use crate::config;
use crate::config::Config;
use crate::runner::Runner;

#[derive(Debug)]
pub struct Project {
    root_dir: PathBuf,
    runners: Vec<Runner>,
}

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("could not find the config file")]
    NoConfigFile,

    #[error("could not load config file")]
    ConfigFileLoad(#[from] config::LoadError),

    #[error("invalid glob pattern found")]
    InvalidGlob(#[from] globset::Error),

    #[error("failed to get the current working directory")]
    CwdNotAccessible,
}

impl Project {
    pub fn load(root_dir: Option<PathBuf>) -> Result<Project, LoadError> {
        // load config file
        let (root_dir, config_file) = Self::find_config(root_dir)?;
        let config = Config::load(&config_file)?;

        trace!("root dir = {:?}", root_dir);
        trace!("config = {:?}", config);

        // build runners for configured formatters
        let formatters: Vec<Runner> = config
            .formatters
            .into_iter()
            .map(|(name, fmt)| Runner::from_formatter(name, fmt))
            .collect::<Result<_, globset::Error>>()?;

        Ok(Project {
            root_dir,
            runners: formatters,
        })
    }

    fn find_config(root_dir: Option<PathBuf>) -> Result<(PathBuf, PathBuf), LoadError> {
        match root_dir {
            Some(root_dir) => {
                // use given dir, ensure it contains the config file
                Config::check_dir(&root_dir).map(|config_file| (root_dir, config_file))
            }
            None => {
                // search from the current working dir upwards
                let cwd = env::current_dir().map_err(|_| LoadError::CwdNotAccessible)?;
                Config::find(&cwd)
            }
        }
        .ok_or(LoadError::NoConfigFile)
    }

    pub fn run(self) -> bool {
        let partitions = self.match_runners();
        let mut all_runners_succeeded = true;
        for (runner, files) in partitions {
            debug!("### {}:\n{:#?}", runner.name(), files);

            let res = runner.run(&self.root_dir, files);
            if let Err(err) = res {
                all_runners_succeeded = false;
                warn!("formatter {} failed to run: {}", runner.name(), err);
            }
        }
        all_runners_succeeded
    }

    /// Walks the whole project directory tree recursively
    /// and partitions it by matching files against the runners' patterns.
    fn match_runners(&self) -> Vec<Partition> {
        // create empty groups for each runner
        let mut partitions: Vec<Partition> = (self.runners.iter())
            .map(|runner| (runner, Vec::new()))
            .collect();

        let git_dir_override = OverrideBuilder::new(&self.root_dir)
            .add("!/.git/")
            .unwrap()
            .build()
            .unwrap();

        // walk the project directory tree
        let walk = WalkBuilder::new(&self.root_dir)
            .overrides(git_dir_override)
            .ignore(false)
            .hidden(false)
            .build();
        for entry in walk {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    warn!("skipping directory entry, cause: {}", error);
                    continue;
                }
            };

            // skip directories
            let entry_type = entry.file_type().expect("entry should not be stdin");
            if entry_type.is_dir() {
                continue;
            }

            let path = entry.path();
            let rel_path = path.strip_prefix(&self.root_dir).unwrap();
            let rel_path_candidate = Candidate::new(rel_path);

            // match each file against the glob sets of each partition's runner:
            //   if the file matches no glob sets, print a warning
            //   if the file matches exactly one glob set, add it to that partition
            //   if the file matches multiple glob sets, print a warning

            let mut matched_runner_files: Option<&mut Vec<PathBuf>> = None;
            for (runner, paths) in &mut partitions {
                let is_match = runner.glob_set().is_match_candidate(&rel_path_candidate);
                if is_match {
                    if matched_runner_files.is_none() {
                        matched_runner_files = Some(paths);
                    } else {
                        warn!(
                            "file {} is already matched by another runner, using only the first",
                            rel_path.display()
                        );
                        break;
                    }
                }
            }

            if let Some(files) = matched_runner_files {
                files.push(entry.into_path());
            } else {
                warn!("file {} is not matched by any runner", rel_path.display());
            }
        }

        partitions
    }
}

type Partition<'a> = (&'a Runner, Vec<PathBuf>);
