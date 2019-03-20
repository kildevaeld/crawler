use super::context::*;
use super::descriptions::*;
use super::environment::Environment;
use super::error::CrawlResult;
use super::work::*;
use conveyor_work::package::Package;
use pathutils;
use slog::Logger;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Target {
    e: Arc<Environment>,
    p: PathBuf,
    d: Arc<TargetDescription>,
}

impl Target {
    pub fn new<P: AsRef<Path>>(
        path: P,
        env: Arc<Environment>,
        description: TargetDescription,
    ) -> CrawlResult<Target> {
        let path = if !path.as_ref().is_absolute() {
            std::fs::canonicalize(path)?
        } else {
            path.as_ref().to_path_buf()
        };

        Ok(Target {
            e: env,
            p: path,
            d: Arc::new(description),
        })
    }

    pub fn env(&self) -> &Environment {
        &self.e
    }

    pub fn path(&self) -> &Path {
        self.p.as_path()
    }

    pub fn description(&self) -> &TargetDescription {
        &self.d
    }

    // pub fn build(self, args: Args) -> CrawlResult<TargetRunner> {
    //     let mut ctx = Context::new(
    //         ParentOrRoot::Root(RootContext::new(self, args)),
    //         None,
    //         None,
    //     );
    //     self.build_with(&mut ctx)
    // }

    // pub fn build_with(self, ctx: &mut Context) -> CrawlResult<TargetRunner> {
    //     let work = self.d.work.build(ctx)?;
    //     Ok(TargetRunner { work: work })
    // }
    pub fn build(self, args: Args) -> CrawlResult<TargetRunner> {
        
        let desc = self.d.clone();
        
        let mut ctx = Context::new(
            ParentOrRoot::Root(RootContext::new(self, args)),
            None,
            None,
        );

        Ok(TargetRunner{
            work: desc.work.build(&mut ctx)?
        })
    }
}

pub struct TargetRunner {
    work: Work<Package>,
}

impl TargetRunner {
    pub async fn run(self) -> CrawlResult<Vec<CrawlResult<Package>>> {
        let worker = Worker::new();
        let ret = await!(worker.run(vec![self.work]));
        Ok(ret)
    }
}
