use super::super::context::Context;
use super::super::error::*;
use super::super::traits::WorkType;
use super::super::utils::station_fn_ctx2;
use super::super::work::{WorkBox, WorkOutput};
use conveyor::{into_box, station_fn};
use conveyor_work::package::Package;
use std::fmt;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone)]
pub struct Duktape {
    pub script: String,
}

impl fmt::Debug for Duktape {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Duktape")
    }
}

#[typetag::serde]
impl WorkType for Duktape {
    fn request_station(&self, ctx: &mut Context) -> CrawlResult<WorkBox<Package>> {
        let log = ctx.log().new(o!("worktype" => "duktape"));

        info!(log, "request duktape station");

        let script = ctx.interpolate(self.script.as_str()).unwrap();

        info!(log, "using script"; "script" => script);

        Ok(into_box(station_fn(async move |mut package: Package| {
            
            Ok(vec![WorkOutput::Result(Ok(package))])
        })))
    }

    fn box_clone(&self) -> Box<WorkType> {
        Box::new(self.clone())
    }
}
