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
pub struct ChildProcess {
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>, 
}

impl fmt::Debug for ChildProcess {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ChildProcess")
    }
}

#[typetag::serde]
impl WorkType for ChildProcess {
    fn request_station(&self, ctx: &mut Context) -> CrawlResult<WorkBox<Package>> {
        let log = ctx.log().new(o!("worktype" => "child-process"));

        info!(log, "request child-process station");

        let command = ctx.interpolate(self.command.as_str()).unwrap();

        info!(log, "using script"; "script" => command);

        Ok(into_box(station_fn(async move |mut package: Package| {
            let body = await!(package.read_content())?;
            Ok(vec![WorkOutput::Result(Ok(package))])
        })))
    }

    fn box_clone(&self) -> Box<WorkType> {
        Box::new(self.clone())
    }
}
