use super::descriptions::WorkTarget;
use super::error::{CrawlErrorKind, CrawlResult};
use super::utils::{interpolate, is_interpolated};
use super::work::WorkBox;
use conveyor_work::package::Package;
use serde_json::Value;
use slog::Logger;
use std::collections::HashMap;
use std::sync::Arc;
pub type Args = HashMap<String, Value>;

#[derive(Clone)]
pub struct Context {
    args: Args,
    logger: Logger,
    root: RootContext,
    parent: Option<Context>,
}

impl Context {
    fn args(&self) -> &Args {
        &self.args
    }
    fn parent(&self) -> Option<&Context> {
        None
    }
    fn interpolate(&self, name: &str) -> Option<String> {}
    fn root(&mut self) -> &mut RootContext {}
    fn log(&self) -> &Logger {}
    fn child(&self) {}
}

#[derive(Clone, Debug)]
pub struct RootContext {
    args: Arc<Args>,
    target: Arc<WorkTarget>,
    logger: Arc<Logger>,
}

impl RootContext {
    pub fn new(logger: &Logger, target: WorkTarget, args: Args) -> RootContext {
        RootContext {
            target: Arc::new(target),
            args: Arc::new(args),
            logger: Arc::new(logger.new(o!("category" => "context" ))),
        }
    }

    pub fn target(&self) -> &WorkTarget {
        &self.target
    }

    pub fn log(&self) -> &Logger {
        &self.logger
    }

    pub fn flow(&mut self, name: &str, args: Args) -> CrawlResult<WorkBox<Package>> {
        let clone = self.target.clone();
        let found = match clone.flows.iter().find(|m| m.name == name) {
            Some(m) => m,
            None => return Err(CrawlErrorKind::NotFound(name.to_string()).into()),
        };

        found.request_station(&args, self)
    }

    fn args(&self) -> &Args {
        &self.args
    }

    fn interpolate(&self, name: &str) -> Option<String> {
        Some(interpolate(name, &self.args))
    }

    fn log(&self) -> &Logger {
        &self.logger
    }
}
