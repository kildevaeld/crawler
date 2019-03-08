use super::super::context::*;
use super::super::error::*;
use super::super::traits::WorkType;
use super::super::utils::*;
use super::super::work::{WorkBox, WorkOutput};
use conveyor::ConveyorError;
use conveyor::*;
use conveyor_http::{Http as WHttp, HttpResponseReader, Url};
use conveyor_work::http::{HttpOptions, Method};
use conveyor_work::prelude::*;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Http {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<Method>,
}

#[typetag::serde]
impl WorkType for Http {
    fn request_station(&self, ctx: &mut Context) -> CrawlResult<WorkBox<Package>> {
        let method = self.method.as_ref().unwrap_or(&Method::GET).clone();

        let http = WHttp::new().pipe(HttpResponseReader);

        Ok(into_box(station_fn_ctx2(
            async move |mut package: Package,
                        ctx: Arc<(Conveyor<WHttp, HttpResponseReader>, Method)>| {
                let body = await!(package.read_content())?;

                let json: String =
                    serde_json::from_slice(&body).map_err(|e| ConveyorError::new(e))?;

                let url = Url::parse(&json).map_err(|e| ConveyorError::new(e))?;
                let options = HttpOptions::new(ctx.1.clone(), url);
                let body = await!(ctx.0.execute(options.to_request()))?;

                Ok(vec![WorkOutput::Result(Ok(package.set_value(body)))])
            },
            Arc::new((http, method)),
        )))
    }

    fn box_clone(&self) -> Box<WorkType> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {

    use super::super::super::prelude::*;
    use super::super::super::utils::WorkBoxWrapper;
    use super::super::super::work;
    use super::*;
    use conveyor_work::http::Method;
    use conveyor_work::prelude::*;
    use serde_json::Value;
    use slog::Logger;
    use tokio;
    pub struct MockContext;

    impl Context for MockContext {
        fn parent(&self) -> Option<&Context> {
            None
        }
        fn interpolate(&self, name: &str) -> Option<String> {
            None
        }

        fn log(&self) -> &Logger {
            unimplemented!("not ");
        }

        fn root(&mut self) -> &mut RootContext {
            unimplemented!("not ");
        }

        fn args(&self) -> &Args {
            unimplemented!("not ");
        }
    }

    #[test]
    fn test_http() {
        tokio::run_async(
            async {
                let http = Http {
                    method: Some(Method::GET),
                };
                let mut ctx = MockContext;
                let station = http.request_station(&mut ctx).unwrap();

                let worker = work::Worker::new();

                // worker.run(vec![Work::new(
                //     Package::new("test", "https://distrowatch.com/"),
                //     WorkBoxWrapper::new(station),
                // )]);

                // let pack =
                //     await!(station.execute(Package::new("test", "https://distrowatch.com/")))
                //         .unwrap();

                let pack = await!(worker.run(vec![work::Work::new(
                    Package::new("test", Value::String("https://distrowatch.com/".to_owned())),
                    WorkBoxWrapper::new(station),
                )]));

                for p in pack {
                    if let Ok(ret) = p {
                        println!("name {}", ret.name());
                    } else {
                        println!("Err {:?}", p.err());
                    }
                }
            },
        );
    }
}
