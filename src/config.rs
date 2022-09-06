use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub formatters: BTreeMap<String, Formatter>,
}

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("failed to read file")]
    ReadFile(#[from] std::io::Error),
    #[error("failed to parse TOML")]
    ParseToml(#[from] toml::de::Error),
}

impl Config {
    /// The possible config file names, ordered by priority.
    pub const FILE_NAMES: [&'static str; 2] = [".forestry.toml", "forestry.toml"];

    /// Loads a config file containing TOML.
    pub fn load(file: &Path) -> Result<Config, LoadError> {
        let file_content = fs::read_to_string(file)?;
        let config = toml::from_str(&file_content)?;
        Ok(config)
    }

    /// Finds the config file in the given directory or any of its parent directories, recursively.
    /// Returns the directory and config file paths or `None`, if not found.
    pub fn find(dir: &Path) -> Option<(PathBuf, PathBuf)> {
        Self::check_dir(dir)
            .map(|file| (dir.to_owned(), file))
            .or_else(|| dir.parent().and_then(Self::find))
    }

    /// Returns the path to a config file in the given directory,
    /// or `None` if the directory does not contain such config file.
    pub fn check_dir(dir: &Path) -> Option<PathBuf> {
        Self::FILE_NAMES.iter().find_map(|&file_name| {
            let file = dir.join(file_name);
            file.is_file().then_some(file)
        })
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Formatter {
    pub program: String,

    #[serde(default)]
    pub args: Vec<String>,

    #[serde(default)]
    pub env: BTreeMap<String, String>,

    pub patterns: Vec<String>,
}

impl Formatter {
    pub fn glob_set(&self) -> Result<GlobSet, globset::Error> {
        let mut glob_set = GlobSetBuilder::new();
        for pattern in &self.patterns {
            glob_set.add(Glob::new(pattern)?);
        }
        glob_set.build()
    }

    /// Returns a new command with the program name, arguments and environment variables preset.
    pub fn new_command(&self) -> Command {
        let mut cmd = Command::new(&self.program);
        cmd.args(&self.args);
        cmd.envs(&self.env);
        cmd
    }
}
