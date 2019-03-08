use super::super::context::{Args, Context, RootContext};
use super::super::error::{CrawlErrorKind, CrawlResult};
use super::super::utils::station_fn_ctx2;
use super::super::utils::{WorkArcWrapper, WorkBoxWrapper};
use super::super::work::{Work, WorkBox, WorkOutput, Worker};
use super::flow_description::*;
use super::work_description::*;
use conveyor::{into_box, Chain};
use conveyor_work::package::Package;
use serde_json::Value;
use slog::{FnValue, Logger};
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkTarget {
    pub name: String,
    pub work: WorkTargetDescription,
    pub flows: Vec<FlowDescription>,
}

pub async fn run_target(
    logger: &Logger,
    target: WorkTarget,
    args: Args,
) -> CrawlResult<Vec<CrawlResult<Package>>> {
    let ctx = RootContext::new(logger, target, args);

    let c = ctx.clone();

    await!(c.target().work.run(ctx))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkTargetDescription {
    pub input: Value,
    pub steps: Vec<WorkDescription>,
}

impl WorkTargetDescription {
    pub async fn run(&self, parent: RootContext) -> CrawlResult<Vec<CrawlResult<Package>>> {
        if self.steps.is_empty() {
            return Ok(vec![]);
        }

        let start = Instant::now();

        let mut ctx = parent.child(&format!("Target({})", parent.target().name), None);

        info!(ctx.log(),"starting target"; "steps" => self.steps.len());

        let mut work = self.steps[0].request_station(&mut ctx).unwrap();
        if self.steps.len() > 1 {
            for w in self.steps.iter().skip(1) {
                let next = w.request_station(&mut ctx)?;
                work = into_box(WorkBoxWrapper::new(work).pipe(station_fn_ctx2(
                    async move |pack: Vec<WorkOutput<Package>>, ctx: Arc<WorkBox<Package>>| {
                        let work = Worker::new();
                        let ret = await!(work.run_chain(pack, ctx.clone()));
                        Ok(ret.into_iter().map(|m| WorkOutput::Result(m)).collect())
                    },
                    Arc::new(next),
                )));
            }
        }

        let worker = Worker::new();
        let ret = await!(worker.run(vec![Work::new(
            Package::new(&parent.target().name, self.input.clone()),
            WorkBoxWrapper::new(work),
        )]));

        info!(ctx.log(), "target finished"; "time" => FnValue(move |_| format!("{:?}",start.elapsed())));

        Ok(ret)
    }
}

#[cfg(test)]
mod tests {

    use super::super::super::prelude::*;
    use super::super::super::utils::WorkBoxWrapper;
    use super::super::super::work;
    use super::*;
    use conveyor::{into_box, station_fn, Station};
    use conveyor_work::http::Method;
    use conveyor_work::prelude::*;
    use serde_json::Value;
    use slog::*;
    use tokio;

    #[test]
    fn test_target() {
        tokio::run_async(
            async {
                let logger = Logger::root(Discard.fuse(), o!());
                let desc = WorkTarget {
                    name: "Loppen".to_string(),
                    work: WorkTargetDescription {
                        input: Value::String("https://loppen.dk".to_string()),
                        steps: vec![WorkDescription {
                            name: Some("description".to_string()),
                            work: Box::new(worktypes::PassThrough {
                                service: Some(Arc::new(into_box(station_fn(
                                    async move |package: Package| {
                                        Ok(vec![work::WorkOutput::Result(Ok(package))])
                                    },
                                )))),
                            }),
                            then: None,
                        }],
                    },
                    flows: vec![],
                };

                let out = await!(run_target(&logger, desc, Args::new())).unwrap();

                for o in out {
                    println!("O {}", o.unwrap().name())
                }
            },
        );
    }
}
