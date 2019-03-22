use super::super::super::context::{Context, ParentOrRoot};
use super::super::super::error::*;
use super::super::super::traits::WorkType;
use super::super::super::utils::station_fn_ctx2;
use super::super::super::work::{WorkBox, WorkOutput};
use conveyor::{into_box, station_fn, WorkStation, Chain};
use conveyor_work::package::Package;
use std::fmt;
use std::sync::{Arc,Mutex};
use super::vm::VM;


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
        let script = ctx.root().resolve_path(script)?;

        info!(log, "using script"; "script" => &script);

        let mut ctx = Context::new(ParentOrRoot::Parent(Box::new(ctx.clone())), None, Some(log));

        let station = station_fn(async move |mut package: Package| {
            let buffer = await!(package.read_content())?;
            Ok((package.name().to_string(), buffer))
        }).pipe(WorkStation::new(2, |package: (String, Vec<u8>), work: &mut VM| {
            info!(work.ctx().log(), "executing script";"script" => &work.script);
            work.run(package)
        }, || VM::new(ctx.clone(), &script)));

        Ok(into_box(station))
        // Ok(into_box(WorkStation::new(1, |package: Package, work: &mut VM| {
        //     info!(work.ctx().log(), "executing script";"script" => &work.script);
        //     work.run(package)
        // }, || VM::new(ctx.clone(), &script))))
    }

    fn box_clone(&self) -> Box<WorkType> {
        Box::new(self.clone())
    }
}
