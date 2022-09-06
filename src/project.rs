use std::env;
use std::path::PathBuf;

use ignore::WalkBuilder;
use log::error;
use thiserror::Error;

use crate::config::{Config, Formatter};

#[derive(Debug)]
pub struct Project {
    root_dir: PathBuf,
    config: Config,
}

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("could not find the config file")]
    NoConfigFile,

    #[error("could not load config file")]
    ConfigFileLoad(#[from] crate::config::LoadError),

    #[error("failed to get the current working directory")]
    CwdNotAccessible,
}

#[derive(Error, Debug)]
pub enum RunError {
    #[error("the file patterns are invalid")]
    InvalidPattern(#[from] globset::Error),
    #[error("failed to iterate the project directory")]
    IterationFailed(#[from] ignore::Error),
    #[error("failed to start the formatter")]
    FormatterStart(std::io::Error),
    #[error("failed to run the formatter")]
    FormatterFailed,
}

impl Project {
    pub fn load(root_dir: Option<PathBuf>) -> Result<Project, LoadError> {
        let (root_dir, config_file) = Self::find_config(root_dir)?;
        let config = Config::load(&config_file)?;
        Ok(Project { root_dir, config })
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
        let mut all_formatters_succeeded = true;
        for (name, fmt) in &self.config.formatters {
            let result = self.run_formatter(fmt);
            if let Err(err) = result {
                error!("formatter '{}' failed:\n{:?}", name, &err);
                all_formatters_succeeded = false;
            }
        }
        all_formatters_succeeded
    }

    pub fn run_formatter(&self, fmt: &Formatter) -> Result<(), RunError> {
        // get iterator of files matching the patterns
        let glob_set = fmt.glob_set()?;
        let files_iter = WalkBuilder::new(&self.root_dir)
            .hidden(false)
            .ignore(false)
            .build()
            .filter_map(|dir_entry| {
                let dir_entry = dir_entry.ok()?;
                let rel_path = dir_entry
                    .path()
                    .strip_prefix(&self.root_dir)
                    .expect("dir entry path should be in project path");
                glob_set.is_match(rel_path).then_some(dir_entry.into_path())
            });

        // build command
        let mut cmd = fmt.new_command();
        cmd.current_dir(&self.root_dir).args(files_iter);

        // run command
        let status = cmd.status().map_err(RunError::FormatterStart)?;
        if status.success() {
            Ok(())
        } else {
            Err(RunError::FormatterFailed)
        }
    }
}
