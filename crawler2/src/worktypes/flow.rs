use super::super::context::{Args, Context};
use super::super::error::*;
use super::super::traits::WorkType;
use super::super::utils::station_fn_ctx2;
use super::super::work::WorkBox;
use conveyor::into_box;
use conveyor_work::package::Package;
use std::fmt;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Flow {
    pub flow_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Args>,
}

#[typetag::serde]
impl WorkType for Flow {
    fn request_station(&self, ctx: &mut Context) -> CrawlResult<WorkBox<Package>> {
        info!(ctx.log().new(o!("worktype" => "flow")),"request flow type station"; "flow_name" => &self.flow_name);
        if let Some(args) = &self.arguments {
            ctx.root().flow(&self.flow_name, args.clone())
        } else {
            ctx.root().flow(&self.flow_name, Args::new())
        }
    }

    fn box_clone(&self) -> Box<WorkType> {
        Box::new(self.clone())
    }
}
