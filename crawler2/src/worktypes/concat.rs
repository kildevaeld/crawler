use super::super::context::{Context, ParentOrRoot};
use super::super::descriptions::{compile_steps, WorkDescription};
use super::super::error::*;
use super::super::traits::WorkType;
use super::super::utils::{station_fn_ctx2, WorkBoxWrapper, WorkArcWrapper};
use super::super::work::{Work, WorkBox, WorkOutput, Worker};
use conveyor::{into_box, station_fn};
use conveyor_work::package::Package;
use std::fmt;
use std::sync::Arc;
use conveyor::futures::prelude::*;
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Concat {
    pub name: String,
    pub steps: Vec<WorkDescription>,
}

#[typetag::serde]
impl WorkType for Concat {
    fn request_station(&self, ctx: &mut Context) -> CrawlResult<WorkBox<Package>> {
        let log = ctx.log().new(o!("worktype" => "concat"));

        info!(log, "request concat station");

        let mut ctx = Context::new(ParentOrRoot::Parent(Box::new(ctx.clone())), None, Some(log));

        let work = compile_steps(&self.steps, &mut ctx)?;

        Ok(into_box(station_fn_ctx2(
            async move |mut package: Package, ctx: Arc<(Context, Arc<WorkBox<Package>>)>| {
                let worker = Worker::new();
                let ret = await!(worker.run(vec![Work::new(package, WorkArcWrapper::new(ctx.1.clone()))]));
                
                if ret.iter().find(|m| m.is_err()).is_some() {
                    return Err(CrawlErrorKind::Unknown.into());
                }
                stream::iter(ret).then(|m| m.read_content()).concat();

                Ok(vec![WorkOutput::Result(Ok(package))])
            },
            Arc::new((ctx, Arc::new(work)),
        )))
    }

    fn box_clone(&self) -> Box<WorkType> {
        Box::new(self.clone())
    }
}
