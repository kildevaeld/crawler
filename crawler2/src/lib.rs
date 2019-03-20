#![feature(async_await, await_macro, futures_api)]

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate typetag;

#[macro_use]
extern crate slog;

#[macro_use]
extern crate lazy_static;

#[macro_use]
#[macro_export]
pub mod macros;
pub mod context;
pub mod descriptions;
pub mod environment;
pub mod error;
pub mod target;
pub mod traits;
pub mod utils;
mod work;
pub mod worktypes;

pub mod prelude {
    pub use super::context::*;
    pub use super::descriptions::*;
    pub use super::environment::*;
    pub use super::target::*;
    pub use super::traits::*;
    pub use super::worktypes;
    pub use serde_json::Value;
}

#[cfg(test)]
mod tests {

    use super::prelude::*;
    use conveyor_work::http::Method;
    use serde_json::Value;
    use serde_yaml;

    #[test]
    fn it_works() {
        let desc = TargetDescription {
            name: "Loppen".to_string(),
            work: WorkTargetDescription {
                input: Value::String("https://loppen.dk".to_string()),
                steps: vec![WorkDescription {
                    name: Some("description".to_string()),
                    work: Box::new(worktypes::Flow {
                        flow_name: "Crawl".to_string(),

                        arguments: Some(args! {
                            "script" => "file://./index.js"
                        }),
                    }),

                    then: None,
                }],
            },
            flows: vec![FlowDescription {
                name: "Crawl".to_string(),
                work: vec![
                    WorkDescription {
                        name: None,
                        work: Box::new(worktypes::Http {
                            method: Some(Method::GET),
                        }),
                        then: None,
                    },
                    WorkDescription {
                        name: None,
                        work: Box::new(worktypes::Duktape {
                            script: "$script".to_string(),
                        }),
                        then: Some(Box::new(worktypes::Flow {
                            flow_name: "Crawl".to_string(),
                            arguments: Some(args! {
                                "script" => "file://./concert.js"
                            }),
                        })),
                    },
                ],
            }],
        };

        let s = serde_yaml::to_string(&desc).unwrap();
        println!("{}", s);

        let desc: TargetDescription = serde_yaml::from_str(&s).unwrap();
        println!("{:?}", desc);
    }
}
