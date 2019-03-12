use super::error::{CrawlErrorKind, CrawlResult};
use super::target::Target;
use super::utils::interpolate;
use super::work::WorkBox;
use conveyor_work::package::Package;
use serde_json::Value;
use slog::{FnValue, Logger};
use std::collections::HashMap;
use std::sync::Arc;
pub type Args = HashMap<String, Value>;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub(crate) enum ParentOrRoot {
    Parent(Box<Context>),
    Root(RootContext),
}

#[derive(Clone, Debug)]
pub struct Context {
    id: Uuid,
    args: Option<Args>,
    logger: Option<Logger>,
    parent: ParentOrRoot,
}

impl Context {
    pub(crate) fn new(parent: ParentOrRoot, args: Option<Args>, logger: Option<Logger>) -> Context {
        Context {
            id: Uuid::new_v4(),
            parent,
            args,
            logger,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
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
        let args = self.all_args();
        info!(self.log(), "interpolate"; "text" => name, "args" => FnValue(|_| serde_json::to_string(&args).unwrap()));

        // let temp = interpolate(name, self.args());
        // match &self.parent {
        //     ParentOrRoot::Parent(p) => p.interpolate(&temp),
        //     ParentOrRoot::Root(r) => r.interpolate(&temp),
        // }
        Some(interpolate(name, &args))
    }

    pub fn interpolate_with(&self, text: &str, args: &Args) -> String {
        let mut oargs = self.all_args();
        for o in args.iter() {
            oargs.insert(o.0.clone(), o.1.clone());
        }
        info!(self.log(), "interpolate"; "text" => text, "args" => FnValue(|_| serde_json::to_string(&oargs).unwrap()));
        interpolate(text, &oargs)
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
                ParentOrRoot::Root(r) => r.target().env().log(),
            },
            Some(s) => &s,
        }
    }

    pub fn child(&self, name: &str, args: Option<Args>) -> Context {
        let id = Uuid::new_v4();
        let logger = self
            .log()
            .new(o!("context" => format!("{}({})", name, id) ));
        Context {
            id: id,
            logger: Some(logger),
            parent: ParentOrRoot::Parent(Box::new(self.clone())),
            args,
        }
    }

    pub fn all_args(&self) -> Args {
        let mut args = match &self.parent {
            ParentOrRoot::Root(r) => r.args().clone(),
            ParentOrRoot::Parent(p) => p.all_args(),
        };

        if let Some(a) = &self.args {
            for e in a {
                args.insert(e.0.clone(), e.1.clone());
            }
        }

        args
    }

    pub fn flow(&mut self, name: &str, args: Args) -> CrawlResult<WorkBox<Package>> {
        let clone = self.root().target().description().clone();
        let found = match clone.flows.iter().find(|m| m.name == name) {
            Some(m) => m,
            None => return Err(CrawlErrorKind::NotFound(name.to_string()).into()),
        };

        //let mut ctx = self.
        found.build(&args, self)
    }
}

#[derive(Clone, Debug)]
struct RootInner {
    id: Uuid,
    args: Args,
    target: Target,
}

#[derive(Clone, Debug)]
pub struct RootContext {
    inner: Arc<RootInner>,
}

impl RootContext {
    pub fn new(target: Target, args: Args) -> RootContext {
        let id = Uuid::new_v4();

        RootContext {
            inner: Arc::new(RootInner {
                id: id,
                target: target,
                args: args,
            }),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.inner.id
    }

    pub fn target(&self) -> &Target {
        &self.inner.target
    }

    pub fn flow(&mut self, name: &str, args: Args) -> CrawlResult<WorkBox<Package>> {
        let clone = self.inner.target.description().clone();
        let found = match clone.flows.iter().find(|m| m.name == name) {
            Some(m) => m,
            None => return Err(CrawlErrorKind::NotFound(name.to_string()).into()),
        };

        let mut ctx = Context::new(ParentOrRoot::Root(self.clone()), None, None);

        found.build(&args, &mut ctx)
    }

    pub fn args(&self) -> &Args {
        &self.inner.args
    }

    pub fn interpolate(&self, name: &str) -> Option<String> {
        debug!(self.target().env().log(), "interpolate"; "text" => name, "args" => FnValue(|_| serde_json::to_string(&self.inner.args).unwrap()));
        Some(interpolate(name, &self.inner.args))
    }

    pub fn child(&self, name: &str, args: Option<Args>) -> Context {
        let id = uuid::Uuid::new_v4();
        Context {
            id: id,
            logger: Some(
                self.target()
                    .env()
                    .log()
                    .new(o!("context" => format!("{}({})",name.to_string(), id))),
            ),
            parent: ParentOrRoot::Root(self.clone()),
            args,
        }
    }
}
