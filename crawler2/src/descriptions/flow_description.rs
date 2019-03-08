use super::super::context::{Args, ChildContext, Context, RootContext, ToChildContext};
use super::super::error::CrawlResult;
use super::super::utils::{interpolate, is_interpolated, station_fn_ctx2};
use super::super::utils::{WorkArcWrapper, WorkBoxWrapper};
use super::super::work::{Work, WorkBox, WorkOutput, Worker};
use conveyor::{into_box, Chain};
use conveyor_work::package::Package;
use slog::Logger;
use std::sync::Arc;

use super::work_description::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlowDescription {
    pub name: String,
    pub work: Vec<WorkDescription>,
}

impl FlowDescription {
    pub fn request_station(&self, args: &Args, ctx: &mut Context) -> CrawlResult<WorkBox<Package>> {
        let logger = ctx.log().new(o!("category" => "flow-context"));

        let mut flow_ctx = FlowContext {
            parent: ctx,
            args,
            logger,
        };

        let mut work = self.work[0].request_station(&mut flow_ctx).unwrap();
        for w in self.work.iter().skip(1) {
            let ww = w.request_station(&mut flow_ctx).unwrap();
            work = into_box(WorkBoxWrapper::new(work).pipe(station_fn_ctx2(
                async move |pack: Vec<WorkOutput<Package>>, ctx: Arc<WorkBox<Package>>| {
                    //
                    let mut v = Vec::new();
                    let mut out = Vec::new();
                    for p in pack {
                        let p = match p {
                            WorkOutput::Result(e) => match e {
                                Ok(o) => Work::new(o, WorkArcWrapper::new(ctx.clone())),
                                Err(e) => {
                                    out.push(Err(e));
                                    continue;
                                }
                            },
                            WorkOutput::Work(e) => e.chain(WorkArcWrapper::new(ctx.clone())),
                            WorkOutput::Then(_) => unimplemented!("not chain"),
                        };
                        v.push(p);
                    }

                    let w = Worker::new();
                    let mut ret = await!(w.run(v));
                    out.extend(ret);
                    Ok(out
                        .into_iter()
                        .map(|m| WorkOutput::Result(m))
                        .collect::<Vec<_>>())
                },
                Arc::new(ww),
            )));
        }
        Ok(work)
    }
}

pub struct FlowContext<'a> {
    parent: &'a mut Context,
    args: &'a Args,
    logger: Logger,
}

impl<'a> Context for FlowContext<'a> {
    fn args(&self) -> &Args {
        self.args
    }

    fn parent(&self) -> Option<&Context> {
        Some(self.parent)
    }
    fn interpolate(&self, name: &str) -> Option<String> {
        self.parent.interpolate(&interpolate(name, self.args))
    }

    fn root(&mut self) -> &mut RootContext {
        self.parent.root()
    }

    fn log(&self) -> &Logger {
        &self.logger
    }
}
