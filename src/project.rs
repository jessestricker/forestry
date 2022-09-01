use std::env;
use std::path::PathBuf;

use thiserror::Error;

use crate::config::Config;

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
}

#[derive(Error, Debug)]
pub enum RunError {}

impl Project {
    pub fn run(&self) {
        todo!()
    }
}
