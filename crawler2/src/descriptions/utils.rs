use super::super::context::Context;
use super::super::error::{CrawlErrorKind, CrawlResult};
use super::super::utils::{station_fn_ctx2, WorkBoxWrapper};
use super::super::work::{WorkBox, WorkOutput, Worker};
use super::WorkDescription;
use conveyor::{into_box, Chain};
use conveyor_work::package::Package;
use slog::FnValue;
use std::sync::Arc;
use std::time::Instant;

pub fn compile_steps(
    steps: &[WorkDescription],
    ctx: &mut Context,
) -> CrawlResult<WorkBox<Package>> {
    if steps.is_empty() {
        return Err(CrawlErrorKind::NotFound("no steps defined".to_string()).into());
    }

    let start = Instant::now();

    info!(ctx.log(),"building target"; "steps" => steps.len());

    let mut work = steps[0].request_station(ctx)?;
    if steps.len() > 1 {
        for w in steps.iter().skip(1) {
            let next = w.request_station(ctx)?;
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

    info!(ctx.log(), "built target"; "time" => FnValue(move |_| format!("{:?}",start.elapsed())));

    Ok(into_box(station_fn_ctx2(
        async move |pack: Package, ctx: Arc<(Context, WorkBox<Package>)>| {
            info!(ctx.0.log(), "target started");

            let now = Instant::now();
            let ret = await!(ctx.1.execute(pack));
            info!(ctx.0.log(), "target executed"; "time" => FnValue(move |_| format!("{:?}",now.elapsed())));
            ret
        },
        Arc::new((ctx.clone(), work)),
    )))
}
