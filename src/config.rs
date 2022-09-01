use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

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
    pub fn find(dir: &Path) -> Option<PathBuf> {
        Self::check_dir(dir).or_else(|| dir.parent().and_then(Self::find))
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
