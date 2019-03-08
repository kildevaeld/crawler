use super::super::context::Context;
use super::super::error::*;
use super::super::traits::WorkType;
use super::super::utils::station_fn_ctx2;
use super::super::work::WorkBox;
use conveyor::into_box;
use conveyor_work::package::Package;
use std::fmt;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone)]
pub struct PassThrough {
    #[serde(skip)]
    pub service: Option<Arc<WorkBox<Package>>>,
}

impl fmt::Debug for PassThrough {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PassThrough")
    }
}

#[typetag::serde]
impl WorkType for PassThrough {
    fn request_station(&self, _ctx: &mut Context) -> CrawlResult<WorkBox<Package>> {
        if self.service.is_none() {
            return Err(CrawlErrorKind::NotFound("inner serice not found".to_string()).into());
        }

        Ok(into_box(station_fn_ctx2(
            move |package: Package, ctx: Arc<WorkBox<Package>>| ctx.execute(package),
            self.service.as_ref().unwrap().clone(),
        )))
    }

    fn box_clone(&self) -> Box<WorkType> {
        Box::new(self.clone())
    }
}
