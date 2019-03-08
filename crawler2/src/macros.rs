#[macro_export]
macro_rules! args {

     { $($key:expr => $value:expr),+ } => {
        {
            use serde_json::{Value,self};
            let mut m = std::collections::HashMap::<String, Value>::new();
            $(
                m.insert($key.to_string(), serde_json::to_value(&$value).unwrap());
            )+
            m
        }
     };
    }
