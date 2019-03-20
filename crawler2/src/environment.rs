use super::context::Args;
use slog::Logger;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct EnvironmentBuilder {
    cwd: PathBuf,
    vars: Args,
    args: Vec<String>,
    log: Logger,
}

impl EnvironmentBuilder {
    pub fn new(cwd: PathBuf, logger: Logger) -> EnvironmentBuilder {
        EnvironmentBuilder {
            cwd: cwd,
            vars: Args::new(),
            args: Vec::new(),
            log: logger,
        }
    }
    pub fn build(self) -> Arc<Environment> {
        Arc::new(Environment {
            cwd: self.cwd,
            vars: self.vars,
            args: self.args,
            logger: self.log,
        })
    }
}

#[derive(Debug)]
pub struct Environment {
    cwd: PathBuf,
    vars: Args,
    args: Vec<String>,
    logger: Logger,
}

impl Environment {
    pub fn build<S: AsRef<Path>>(cwd: S, logger: Logger) -> EnvironmentBuilder {
        EnvironmentBuilder::new(cwd.as_ref().to_path_buf(), logger)
    }

    pub fn cwd(&self) -> &Path {
        self.cwd.as_path()
    }

    pub fn vars(&self) -> &Args {
        &self.vars
    }

    pub fn log(&self) -> &Logger {
        &self.logger
    }
}
