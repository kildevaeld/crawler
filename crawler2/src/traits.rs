use super::context::Context;
use super::descriptions::WorkTarget;
use super::error::{CrawlError, CrawlErrorKind, CrawlResult};
use super::work::*;
use conveyor::futures::prelude::*;
use conveyor::{Result as StationResult, Station};
use conveyor_work::{package::Package, utils::BoxedStation};
use serde;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::pin::Pin;
// pub type Args = HashMap<String, Value>;

// pub trait Context {
//     fn parent(&self) -> Option<&Context>;
//     fn interpolate(&self, name: &str) -> String;

//     fn root(&mut self) -> &mut Context;

//     // fn target(&mut self) -> &WorkTarget {
//     //     self.root().target()
//     // }

//     fn flow(&mut self, name: &str, args: Args) -> CrawlResult<WorkBox<Package>>;
// }

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
