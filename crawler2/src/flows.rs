use super::descriptions::*;
use super::context::Args;
pub struct Flows {
    target: Arc<TargetDescription>
}


impl Flows {
    pub fn new(target: Arc<TargetDescription>) -> Flows {
        Flows{target}
    }

    pub fn request<S: AsRef<str>>(&self, name: S) {
        
    }
}