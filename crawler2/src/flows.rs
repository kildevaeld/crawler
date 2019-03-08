use super::descriptions::*;
use super::context::Args;
pub struct Flows {
    target: Arc<WorkTarget>
}


impl Flows {
    pub fn new(target: Arc<WorkTarget>) -> Flows {
        Flows{target}
    }

    pub fn request<S: AsRef<str>>(&self, name: S) {
        
    }
}