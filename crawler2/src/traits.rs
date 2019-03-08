use super::context::Context;
use super::error::CrawlResult;
use super::work::*;
use conveyor_work::package::Package;
use std::fmt;

#[typetag::serde(tag = "type")]
pub trait WorkType: fmt::Debug + Send + Sync {
    fn request_station(&self, ctx: &mut Context) -> CrawlResult<WorkBox<Package>>;
    fn box_clone(&self) -> Box<WorkType>;
}

impl Clone for Box<WorkType> {
    fn clone(&self) -> Box<WorkType> {
        self.box_clone()
    }
}
