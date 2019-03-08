use super::error::{CrawlErrorKind, CrawlResult};
use super::utils::{station_fn_ctx2, WorkArcWrapper, WorkBoxWrapper};
use conveyor::futures::prelude::*;
use conveyor::Chain;
use conveyor::ConcurrentStream;
use conveyor::{into_box, Result, Station};
use conveyor_work::prelude::*;
use std::pin::Pin;
use std::sync::Arc;

pub enum WorkOutput<V: 'static + Send> {
    Result(CrawlResult<V>),
    Work(Work<V>),
    Then(V),
}

pub type WorkBox<V> = Box<
    Station<
            Input = V,
            Output = Vec<WorkOutput<V>>,
            Future = Pin<Box<Future<Output = Result<Vec<WorkOutput<V>>>> + Send>>,
        > + Send
        + Sync,
>;

pub struct Work<V: 'static + Send> {
    data: V,
    work: WorkBox<V>,
}

impl<V: Send + Sync> Work<V> {
    pub fn new<
        W: Station<Input = V, Output = Vec<WorkOutput<V>>, Future = F> + 'static + Send + Sync,
        F: Future<Output = Result<Vec<WorkOutput<V>>>> + Send,
    >(
        data: V,
        work: W,
    ) -> Work<V> {
        Work {
            data,
            work: into_box(work),
        }
    }

    pub fn chain<
        W: Station<Input = V, Output = Vec<WorkOutput<V>>, Future = F> + 'static + Send + Sync,
        F: Future<Output = Result<Vec<WorkOutput<V>>>> + Send,
    >(
        self,
        next: W,
    ) -> Work<V> {
        Work {
            data: self.data,
            work: into_box(WorkBoxWrapper::new(self.work).pipe(station_fn_ctx2(
                async move |pack: Vec<WorkOutput<V>>, ctx: Arc<WorkBox<V>>| {
                    let (mut ret, w, mut thens) = Worker::split(pack);

                    let worker = Worker::new();
                    let out = await!(worker.run(w));
                    ret.extend(out);

                    let (mut ret, w) = Worker::split2(ret, ctx.clone());
                    let out = await!(worker.run(w));
                    ret.extend(out);
                    let mut out = ret
                        .into_iter()
                        .map(|m| WorkOutput::Result(m))
                        .collect::<Vec<_>>();
                    out.extend(
                        thens
                            .into_iter()
                            .map(|m| WorkOutput::Then(m))
                            .collect::<Vec<_>>(),
                    );

                    Ok(out)
                },
                Arc::new(into_box(next)),
            ))),
        }
    }
}

pub struct Worker;

impl Worker {
    pub fn new() -> Worker {
        Worker
    }

    fn _run<'a, V: 'static + Send>(
        &'a self,
        input: Vec<Work<V>>,
    ) -> impl Future<Output = Vec<Result<Vec<WorkOutput<V>>>>> {
        let stream = stream::iter(input);
        ConcurrentStream::new(stream.map(|work| work.work.execute(work.data)), 4).collect()
    }

    pub async fn run<V: 'static + Send>(&self, input: Vec<Work<V>>) -> Vec<CrawlResult<V>> {
        let mut ret = input;
        let mut output = Vec::new();
        loop {
            ret = await!(self._run(ret))
                .into_iter()
                .filter_map(|m| match m {
                    Ok(s) => Some(
                        s.into_iter()
                            .filter_map(|m| match m {
                                WorkOutput::Result(r) => {
                                    output.push(r);
                                    None
                                }
                                WorkOutput::Work(w) => Some(w),
                                WorkOutput::Then(_) => unimplemented!("cannot chain"),
                            })
                            .collect::<Vec<_>>(),
                    ),
                    Err(e) => {
                        output.push(Err(CrawlErrorKind::Conveyor(e.to_string()).into()));
                        None
                    }
                })
                .flatten()
                .collect();

            if ret.is_empty() {
                break;
            }
        }
        output
    }

    // pub async fn run2<V: 'static + Send>(&self, input: Vec<WorkOutput<V>>) -> Vec<CrawlResult<V>> {
    //     let (mut output, mut ret) = Worker::split(input);
    //     //let mut output = Vec::new();
    //     loop {
    //         ret = await!(self._run(ret))
    //             .into_iter()
    //             .filter_map(|m| match m {
    //                 Ok(s) => Some(
    //                     s.into_iter()
    //                         .filter_map(|m| match m {
    //                             WorkOutput::Result(r) => {
    //                                 output.push(r);
    //                                 None
    //                             }
    //                             WorkOutput::Work(w) => Some(w),
    //                         })
    //                         .collect::<Vec<_>>(),
    //                 ),
    //                 Err(e) => {
    //                     output.push(Err(CrawlErrorKind::Unknown.into()));
    //                     None
    //                 }
    //             })
    //             .flatten()
    //             .collect();

    //         if ret.is_empty() {
    //             break;
    //         }
    //     }
    //     output
    // }

    pub async fn run_and<V: 'static + Send + Sync>(
        &self,
        input: Vec<WorkOutput<V>>,
        work: Arc<WorkBox<V>>,
    ) -> Vec<CrawlResult<V>> {
        let (mut output, mut ret, mut thens) = Worker::split(input);
        loop {
            ret = await!(self._run(ret))
                .into_iter()
                .filter_map(|m| match m {
                    Ok(s) => Some(
                        s.into_iter()
                            .filter_map(|m| match m {
                                WorkOutput::Result(r) => {
                                    output.push(r);
                                    None
                                }
                                WorkOutput::Work(w) => Some(w),
                                WorkOutput::Then(r) => {
                                    thens.push(r);
                                    None
                                }
                            })
                            .collect::<Vec<_>>(),
                    ),
                    Err(e) => {
                        output.push(Err(CrawlErrorKind::Conveyor(e.to_string()).into()));
                        None
                    }
                })
                .flatten()
                .collect();

            if ret.is_empty() {
                break;
            }
        }

        let (mut output, ret) = Worker::split2(thens.into_iter().map(|m| Ok(m)).collect(), work);
        println!("len {} {}", output.len(), ret.len());
        let ret = await!(self.run(ret));
        output.extend(ret);

        output
    }

    pub async fn run_chain<V: 'static + Send + Sync>(
        &self,
        input: Vec<WorkOutput<V>>,
        work: Arc<WorkBox<V>>,
    ) -> Vec<CrawlResult<V>> {
        let (mut output, mut ret, _) = Worker::split(input);
        loop {
            ret = await!(self._run(ret))
                .into_iter()
                .filter_map(|m| match m {
                    Ok(s) => Some(
                        s.into_iter()
                            .filter_map(|m| match m {
                                WorkOutput::Result(r) => {
                                    output.push(r);
                                    None
                                }
                                WorkOutput::Work(w) => Some(w),
                                WorkOutput::Then(r) => {
                                    output.push(Ok(r));
                                    None
                                }
                            })
                            .collect::<Vec<_>>(),
                    ),
                    Err(e) => {
                        output.push(Err(CrawlErrorKind::Conveyor(e.to_string()).into()));
                        None
                    }
                })
                .flatten()
                .collect();

            if ret.is_empty() {
                break;
            }
        }

        let (mut output, ret) = Worker::split2(output, work);

        let ret = await!(self.run(ret));
        output.extend(ret);

        output
    }

    pub fn split<V: 'static + Send>(
        input: Vec<WorkOutput<V>>,
    ) -> (Vec<CrawlResult<V>>, Vec<Work<V>>, Vec<V>) {
        let mut output = Vec::new();
        let mut thens = Vec::new();
        let ret = input
            .into_iter()
            .filter_map(|m| match m {
                WorkOutput::Work(w) => Some(w),
                WorkOutput::Result(ret) => {
                    output.push(ret);
                    None
                }
                WorkOutput::Then(ret) => {
                    thens.push(ret);
                    None
                }
            })
            .collect();

        (output, ret, thens)
    }

    pub fn split2<V: 'static + Send + Sync>(
        input: Vec<CrawlResult<V>>,
        work: Arc<WorkBox<V>>,
    ) -> (Vec<CrawlResult<V>>, Vec<Work<V>>) {
        let mut output = Vec::new();
        let ret = input
            .into_iter()
            .filter_map(|m| match m {
                Ok(m) => Some(Work::new(m, WorkArcWrapper::new(work.clone()))),
                Err(e) => {
                    output.push(Err(e));
                    None
                }
            })
            .collect();

        (output, ret)
    }

    // pub async fn split_ret<V: 'static + Send>(
    //     input: Vec<Result<WorkOutput<V>>>,
    // ) -> (Vec<CrawlResult<V>>, Vec<Work<V>>) {
    //     let mut output = Vec::new();
    //     let ret = input
    //         .into_iter()
    //         .filter_map(|m| match m {
    //             Ok(s) => match s {
    //                 WorkOutput::Result(ret) => {
    //                     output.push(ret);
    //                     None
    //                 }
    //                 WorkOutput::Work(w) => Some(w),
    //             },
    //             Err(e) => {
    //                 output.push(Err(CrawlErrorKind::Unknown.into()));
    //                 None
    //             }
    //         })
    //         .collect();

    //     (output, ret)
    // }
}

#[cfg(test)]
mod tests {

    use super::*;
    use conveyor::futures;
    use conveyor::{station_fn, Station};

    #[test]
    fn it_works() {
        let work = Work::new(
            String::from("Value, baby!"),
            station_fn(async move |val| Ok(vec![WorkOutput::Result(Ok(val))])),
        );

        let worker = Worker::new();

        let ret = futures::executor::block_on(worker.run(vec![work]));

        assert_eq!(ret.len(), 1);
        assert!(ret[0].is_ok());
        //assert_eq!(&ret[0].unwrap(), String::from("Value, baby!"));
    }

    #[test]
    fn more_work() {
        let work = Work::new(
            String::from("Value, baby!"),
            station_fn(async move |val: String| {
                Ok(vec![
                    WorkOutput::Result(Ok(val)),
                    WorkOutput::Work(Work::new(
                        String::from("Value, baby 2!"),
                        station_fn(async move |val| Ok(vec![WorkOutput::Result(Ok(val))])),
                    )),
                ])
            }),
        );

        let worker = Worker::new();

        let ret = futures::executor::block_on(worker.run(vec![work]));

        assert_eq!(ret.len(), 2);
        //assert_eq!(&ret[0].unwrap(), String::from("Value, baby!"));
    }

    // #[test]
    // fn tokio_work() {
    //     tokio::run_async(
    //         async {
    //             let http = Http::new();

    //             let worker = Worker::new();

    //             let ret = await!(worker.run(vec![Work::new(
    //                 "https://distrowatch.com".to_string(),
    //                 station_fn(async move |val: String| {
    //                     let url = Url::parse(val.as_str()).unwrap();
    //                     let req = Request::new(Method::GET, url);
    //                     let result = await!(Http::new()
    //                         .chain(HttpResponseReader)
    //                         .chain(utils::to_string())
    //                         .execute(req))?;
    //                     Ok(vec![WorkOutput::Result(result)])
    //                 }),
    //             )]));

    //             for r in ret {
    //                 println!("{:?}", r);
    //             }
    //         },
    //     );
    // }

}
