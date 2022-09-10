use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub global: GlobalConfig,
    pub formatters: BTreeMap<String, FormatterConfig>,
}

#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct GlobalConfig {
    #[serde(default)]
    pub ignores: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct FormatterConfig {
    pub program: String,

    #[serde(default)]
    pub shell: bool,

    #[serde(default)]
    pub args: Vec<String>,

    #[serde(default)]
    pub env: BTreeMap<String, String>,

    pub patterns: Vec<String>,
}

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("failed to read file")]
    ReadFailed(#[from] std::io::Error),

    #[error("file contains invalid TOML")]
    InvalidToml(#[from] toml::de::Error),
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
