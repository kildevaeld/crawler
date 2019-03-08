use super::descriptions::WorkTarget;
use super::error::{CrawlErrorKind, CrawlResult};
use super::utils::{interpolate, is_interpolated};
use super::work::WorkBox;
use conveyor_work::package::Package;
use serde_json::Value;
use slog::{FnValue, Logger};
use std::collections::HashMap;
use std::sync::Arc;
pub type Args = HashMap<String, Value>;

#[derive(Clone, Debug)]
pub(crate) enum ParentOrRoot {
    Parent(Box<Context>),
    Root(RootContext),
}

#[derive(Clone, Debug)]
pub struct Context {
    args: Option<Args>,
    logger: Option<Logger>,
    parent: ParentOrRoot,
}

impl Context {
    pub(crate) fn new(parent: ParentOrRoot, args: Option<Args>, logger: Option<Logger>) -> Context {
        Context {
            parent,
            args,
            logger,
        }
    }

    pub fn args(&self) -> &Args {
        match &self.args {
            None => match &self.parent {
                ParentOrRoot::Parent(p) => p.args(),
                ParentOrRoot::Root(r) => r.args(),
            },
            Some(s) => &s,
        }
    }

    pub fn parent(&self) -> Option<&Context> {
        match &self.parent {
            ParentOrRoot::Parent(p) => Some(p),
            ParentOrRoot::Root(_) => None,
        }
    }

    pub fn interpolate(&self, name: &str) -> Option<String> {
        debug!(self.log(), "interpolate"; "text" => name, "args" => FnValue(|_| serde_json::to_string(self.args()).unwrap()));

        let temp = interpolate(name, self.args());
        match &self.parent {
            ParentOrRoot::Parent(p) => p.interpolate(&temp),
            ParentOrRoot::Root(r) => r.interpolate(&temp),
        }
    }

    pub fn root(&mut self) -> &mut RootContext {
        match &mut self.parent {
            ParentOrRoot::Parent(p) => p.root(),
            ParentOrRoot::Root(r) => r,
        }
    }

    pub fn log(&self) -> &Logger {
        match &self.logger {
            None => match &self.parent {
                ParentOrRoot::Parent(p) => p.log(),
                ParentOrRoot::Root(r) => r.log(),
            },
            Some(s) => &s,
        }
    }

    pub fn child(&self, name: &str, args: Option<Args>) -> Context {
        Context {
            logger: Some(self.log().new(o!("context" => name.to_string()))),
            parent: ParentOrRoot::Parent(Box::new(self.clone())),
            args,
        }
    }
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
            logger: Arc::new(logger.new(o!("context" => "root" ))),
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

        let mut ctx = Context::new(ParentOrRoot::Root(self.clone()), None, None);

        found.request_station(&args, &mut ctx)
    }

    pub fn args(&self) -> &Args {
        &self.args
    }

    pub fn interpolate(&self, name: &str) -> Option<String> {
        debug!(self.logger, "interpolate"; "text" => name, "args" => FnValue(|_| serde_json::to_string(self.args.as_ref()).unwrap()));
        Some(interpolate(name, &self.args))
    }

    pub fn child(&self, name: &str, args: Option<Args>) -> Context {
        Context {
            logger: Some(self.log().new(o!("context" => name.to_string()))),
            parent: ParentOrRoot::Root(self.clone()),
            args,
        }
    }
}
