use super::super::context::{Context, ParentOrRoot};
use super::super::descriptions::{compile_steps, WorkDescription};
use super::super::error::*;
use super::super::traits::WorkType;
use super::super::utils::{station_fn_ctx2, WorkArcWrapper, WorkBoxWrapper};
use super::super::work::{Work, WorkBox, WorkOutput, Worker};
use conveyor::futures::prelude::*;
use conveyor::{into_box, station_fn};
use conveyor_work::package::{ConcatStream, Package};
use std::fmt;
use std::sync::Arc;


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
            async move |mut package: Package,
                        ctx: Arc<(Context, Arc<WorkBox<Package>>, String)>| {
                info!(ctx.0.log(), "running concat");
                let worker = Worker::new();
                let ret = await!(
                    worker.run(vec![Work::new(package, WorkArcWrapper::new(ctx.1.clone()))])
                );

                // if ret.iter().find(|m| m.is_err()).is_some() {
                //     return Err(CrawlErrorKind::Unknown.into());
                // }

                let name = ctx.0.interpolate(&ctx.2).unwrap();

                let s = await!(ConcatStream::new(
                    stream::iter(ret)
                        .then(async move |mut m| await!(m.as_mut().unwrap().read_content()))
                ))?;

                Ok(vec![WorkOutput::Result(Ok(Package::new(&name, s)))])
            },
            Arc::new((ctx, Arc::new(work), self.name.clone())),
        )))
    }

    fn box_clone(&self) -> Box<WorkType> {
        Box::new(self.clone())
    }
}
