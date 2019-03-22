use super::environment::*;
use super::target::*;
use super::error::CrawlResult;
use std::sync::Arc;
use std::path::Path;
use std::future::Future;
use super::context::Args;

#[derive(Debug)]
pub struct Engine {
    targets: Vec<Target>,
    env: Arc<Environment>
}

impl Engine {
    pub fn new(env: Arc<Environment>) -> Engine {
        Engine{
            targets: Vec::new(),
            env: env
        }
    }

    pub fn add_target(&mut self, target: Target) -> &mut Self {
        self.targets.push(target);
        self
    }

    pub fn add_file<P: AsRef<Path>>(&mut self, path: P) -> CrawlResult<&mut Self> {
        let target = Target::from_file(path, self.env.clone())?;
        self.targets.push(target);
        Ok(self)
    }


    pub async fn run<S: AsRef<str> + 'static>(&self, name: S, args: Args) -> CrawlResult<()> {
        let name = name.as_ref();
        let p = match self.targets.iter().find(|m| m.description().name.as_str() == name) {
            Some(p) => p.clone().build(args)?,
            None => unimplemented!("not found")
        };

        let ret = await!(p.run())?;

        Ok(())
    }

}
