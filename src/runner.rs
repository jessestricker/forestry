use std::cmp::Ordering;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

use globset::{Glob, GlobSet, GlobSetBuilder};
use log::trace;
use thiserror::Error;

use crate::config::FormatterConfig;

#[derive(Debug)]
pub struct Runner {
    name: String,
    glob_set: GlobSet,

    program: String,
    shell: bool,
    args: Vec<String>,
    env: HashMap<String, String>,
}

impl PartialEq<Self> for Runner {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl Eq for Runner {}

impl PartialOrd<Self> for Runner {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Ord for Runner {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ExecFailed(std::io::Error),

    #[error("executed program did not exit successfully")]
    ProgramFailed,
}

impl Runner {
    /// Create a new runner by consuming a formatter.
    pub fn from_formatter(name: String, fmt: FormatterConfig) -> Result<Self, globset::Error> {
        let runner = Self {
            name,
            glob_set: Self::build_glob_set(fmt.patterns)?,
            program: fmt.program,
            shell: fmt.shell,
            args: fmt.args,
            env: fmt.env.into_iter().collect(),
        };
        Ok(runner)
    }

    fn build_glob_set<I: IntoIterator<Item = S>, S: AsRef<str>>(
        patterns: I,
    ) -> Result<GlobSet, globset::Error> {
        let mut builder = GlobSetBuilder::new();
        for pattern in patterns {
            builder.add(Glob::new(pattern.as_ref())?);
        }
        builder.build()
    }

    /// Executes the runner once in a specific directory for a set of paths.
    pub fn run<I, S>(&self, working_dir: &Path, paths: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        // build command
        let mut cmd = if self.shell {
            self.new_shell_cmd()
        } else {
            Command::new(&self.program)
        };
        cmd.current_dir(working_dir);
        cmd.envs(&self.env);
        cmd.args(&self.args);
        cmd.args(paths);

        // execute command
        trace!("(runner {}) executing {:?}", self.name, cmd);
        let status = cmd.status().map_err(Error::ExecFailed)?;

        // inspect command result
        trace!("(runner {}) status of last command {:?}", self.name, status);
        if !status.success() {
            Err(Error::ProgramFailed)
        } else {
            Ok(())
        }
    }

    #[cfg(windows)]
    fn new_shell_cmd(&self) -> Command {
        let mut cmd = Command::new("cmd.exe");
        cmd.args(["/c", &self.program]);
        cmd
    }

    #[cfg(unix)]
    fn new_shell_cmd(&self) -> Command {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", &self.program]);
        cmd
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn glob_set(&self) -> &GlobSet {
        &self.glob_set
    }
    pub fn program(&self) -> &str {
        &self.program
    }
    pub fn args(&self) -> &Vec<String> {
        &self.args
    }
    pub fn env(&self) -> &HashMap<String, String> {
        &self.env
    }
}
