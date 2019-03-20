use super::super::context::{Context, ParentOrRoot};
use super::super::error::*;
use super::super::traits::WorkType;
use super::super::utils::station_fn_ctx2;
use super::super::work::{WorkBox, WorkOutput};
use conveyor::{futures::prelude::*, into_box, station_fn, Result, Station, WorkStation};

use conveyor_work::package::Package;
use pathutils;
use std::fmt;
use std::sync::Arc;
use vfs::{VPath, WritePath, VFS};
use std::io::Write;

#[derive(Serialize, Deserialize, Clone)]
pub struct WriteDirectory {
    pub path: String,
}

impl fmt::Debug for WriteDirectory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WriteDirectory")
    }
}

#[typetag::serde]
impl WorkType for WriteDirectory {
    fn request_station(&self, ctx: &mut Context) -> CrawlResult<WorkBox<Package>> {
        let log = ctx.log().new(o!("worktype" => "write-directory"));

        info!(log, "request write-directory station");

        let mut path = ctx.interpolate(self.path.as_str()).unwrap();

        if !pathutils::is_absolute(&path) {
            path = pathutils::to_absolute(&path, ctx.root().target().path().to_str().unwrap())
                .map_err(|_| CrawlError::new(CrawlErrorKind::Unknown))?;
        }

        info!(log, "using path"; "path" => &path);

        if !std::path::Path::new(&path).exists() {
            return Err(CrawlErrorKind::NotFound("path does not exits".to_string()).into());
        }

        let ctx = Context::new(ParentOrRoot::Parent(Box::new(ctx.clone())), None, Some(log));

        Ok(into_box(WorkStation::new(
            1,
            |mut package: Package, ctx: &mut (vfs::physical::PhysicalFS, Context)| {
                let path = ctx.0.path(package.name());
                if path.exists() {
                    return Ok(vec![WorkOutput::Result(Ok(package))]);
                }

                info!(ctx.1.log(), "writing path"; "path" => format!("{:?}", path));

                let buf = conveyor::futures::executor::block_on(package.read_content())?;

                let mut file = path.create().unwrap();
                file.write(&buf);
                file.flush();

                Ok(vec![WorkOutput::Result(Ok(package))])
            },
            move || (vfs::physical::PhysicalFS::new(&path).unwrap(), ctx.clone()),
        )))
    }

    fn box_clone(&self) -> Box<WorkType> {
        Box::new(self.clone())
    }
}
