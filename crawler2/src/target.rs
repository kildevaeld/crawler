use super::context::*;
use super::descriptions::*;
use super::environment::Environment;
use super::error::{CrawlResult, CrawlErrorKind, CrawlError};
use super::work::*;
use conveyor_work::package::Package;
use pathutils;
use slog::Logger;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use serde_json;
use std::fs;

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

    pub fn from_file<P: AsRef<Path>>(path: P, env: Arc<Environment>) -> CrawlResult<Target> {

        if !path.as_ref().exists() || path.as_ref().is_dir() {
            return Err(CrawlErrorKind::InvalidDescriptionFile(path.as_ref().to_path_buf()).into());
        }

        let ext = match path.as_ref().extension() {
            Some(e) => e.to_str().unwrap(),
            None => return Err(CrawlErrorKind::InvalidDescriptionFile(path.as_ref().to_path_buf()).into())
        };

        let parent = path.as_ref().parent().unwrap_or(Path::new("/"));

        let mut file = fs::File::open(&path)?;

        let desc = match ext {
            "json" => serde_json::from_reader(file).map_err(|e| CrawlError::new(CrawlErrorKind::Error(Box::new(e)))),
            "yaml" | "yml" => serde_yaml::from_reader(file).map_err(|e| CrawlError::new(CrawlErrorKind::Error(Box::new(e)))),
            _ => Err(CrawlError::new(CrawlErrorKind::InvalidDescriptionFile(path.as_ref().to_path_buf()))),
        }?;

        
        Target::new(parent, env, desc)
        
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
