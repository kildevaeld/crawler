use super::super::context::Context;
use super::super::error::CrawlResult;
use super::super::traits::WorkType;
use super::super::utils::station_fn_ctx2;
use super::super::work::{WorkBox, WorkOutput, Worker};
use conveyor::into_box;
use conveyor_work::package::Package;

use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkDescription {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(flatten)]
    pub work: Box<WorkType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub then: Option<Box<WorkType>>,
}

impl WorkDescription {
    pub fn request_station(&self, ctx: &mut Context) -> CrawlResult<WorkBox<Package>> {
        let ret = (
            self.work.request_station(ctx)?,
            match &self.then {
                Some(m) => Some(Arc::new(m.request_station(ctx)?)),
                None => None,
            },
        );

        Ok(into_box(station_fn_ctx2(
            async move |pack: Package,
                        ctx: Arc<(WorkBox<Package>, Option<Arc<WorkBox<Package>>>)>| {
                let mut ret = await!(ctx.0.execute(pack))?;
                if let Some(then) = &ctx.1 {
                    let worker = Worker::new();
                    let output = await!(worker.run_and(ret, then.clone()));
                    ret = output.into_iter().map(|m| WorkOutput::Result(m)).collect();
                }

                Ok(ret)
            },
            Arc::new(ret),
        )))
    }
}
