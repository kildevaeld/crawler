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
                Some(s) => Some(s.clone()),
                None => None,
            },
            ctx.clone(),
        );

        Ok(into_box(station_fn_ctx2(
            async move |pack: Package,
                        ctx: Arc<(WorkBox<Package>, Option<Box<WorkType>>, Context)>| {
                let mut ret = await!(ctx.0.execute(pack))?;

                if ret.iter().find(|m| m.is_then()).is_some() {
                    if let Some(then) = &ctx.1 {
                        let mut c = ctx.2.clone();
                        let worker = Worker::new();
                        let output =
                            await!(worker
                                .run_and(ret, Arc::new(then.request_station(&mut c).unwrap())));
                        ret = output.into_iter().map(|m| WorkOutput::Result(m)).collect();
                    }
                }

                Ok(ret)
            },
            Arc::new(ret),
        )))
    }
}
