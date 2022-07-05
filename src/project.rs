use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Error;
use serde::Deserialize;

/// A project directory tree in which files can be formatted/linted.
#[derive(Debug)]
pub struct Project {
    /// The project root directory.
    pub root_dir: PathBuf,

    /// The project configuration.
    pub config: ProjectConfig,
}

impl Project {
    /// The name of the project configuration file.
    pub const CONFIG_FILE_NAME: &'static str = "forestry.toml";

    /// Loads the nearest project.
    ///
    /// This function searches for the project configuration file,
    /// starting with the current working directory
    /// and, if not found, checking any parent directory.
    ///
    /// Returns [`None`] if no project was found.
    pub fn load_nearest() -> Result<Option<Self>, Error> {
        // find nearest config file
        let start_dir = env::current_dir()?;

        let mut project_dir = start_dir.as_path();
        let mut config_file: PathBuf;
        loop {
            config_file = project_dir.join(Self::CONFIG_FILE_NAME);
            if config_file.is_file() {
                break;
            }

            // try parent dir
            project_dir = if let Some(parent) = project_dir.parent() {
                parent
            } else {
                // no config file exists
                return Ok(None);
            }
        }

        // load config file, construct project
        let config = ProjectConfig::load_from_file(&config_file)?;
        let project = Self {
            root_dir: project_dir.to_path_buf(),
            config,
        };
        Ok(Some(project))
    }
}

/// The project configuration.
///
/// It contains a description of the tools to be run.
/// During the initialization phase, this struct is loaded from a
/// file in the project root directory.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ProjectConfig {
    /// A collection of named formatters.
    pub formatters: HashMap<String, FormatterConfig>,
}

impl ProjectConfig {
    /// Load a project configuration from a TOML file.
    pub fn load_from_file(path: &Path) -> anyhow::Result<ProjectConfig> {
        let content = fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
}

/// A configuration of a code formatting tool.
///
/// It describes where to find the files and how to build a command line
/// to execute the program.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormatterConfig {
    /// The program to be executed.
    ///
    /// This can either be a path to an executable file or a command name
    /// which will be resolved using the `PATH` environment variable.
    pub program: String,

    /// The sequence of arguments to pass to the program.
    #[serde(default, rename = "args")]
    pub arguments: Vec<String>,

    /// A list of file glob patterns, relative to the project root directory.
    /// The resolved file paths will be appended to the command line.
    #[serde(rename = "files")]
    pub file_patterns: Vec<String>,
}
