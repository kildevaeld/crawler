use super::task::{ParseTask, Task};
#[macro_use]
use super::utils::measure;
use super::error::{ErrorKind, Result};
use super::vm::VM;
use crossbeam;
use duktape::prelude::*;
use duktape::Context;
use duktape_modules::{self, CJSContext};
use duktape_stdlib;
use reqwest::{self, Client};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use value::Value;

use crossbeam_channel::{after, bounded, unbounded, Receiver, Sender};

pub struct CrawlerConfig {
    producers: u32,
    consumers: u32,
    workers: u32,
    timeout: Duration,
    buffer_size: usize,
    output: Option<String>,
}

impl CrawlerConfig {
    pub fn producers(&mut self, n: u32) -> &mut Self {
        self.producers = n;
        self
    }

    pub fn consumers(&mut self, n: u32) -> &mut Self {
        self.consumers = n;
        self
    }

    pub fn workers(&mut self, n: u32) -> &mut Self {
        self.workers = n;
        self
    }

    pub fn timeout(&mut self, timeout: Duration) -> &mut Self {
        self.timeout = timeout;
        self
    }

    pub fn buffer_size(&mut self, buffer_size: usize) -> &mut Self {
        self.buffer_size = buffer_size;
        self
    }

    pub fn output<T: AsRef<str>>(&mut self, output: T) -> &mut Self {
        self.output = Some(output.as_ref().to_string());
        self
    }

    pub fn build(self) -> Crawler {
        Crawler {
            queue: vec![],
            running: false,
            cfg: self,
        }
    }
}

impl Default for CrawlerConfig {
    fn default() -> CrawlerConfig {
        CrawlerConfig {
            producers: 1,
            consumers: 2,
            timeout: Duration::from_secs(5),
            buffer_size: 10,
            workers: 4,
            output: None,
        }
    }
}

pub struct Crawler {
    queue: Vec<Task>,
    running: bool,
    cfg: CrawlerConfig,
}

fn download(client: &Client, id: u32, task: &Task) -> Result<String> {
    //let (elapsed, res) = measure(|| client.get(task.url().as_str()).send());

    let mut res = client.get(task.url().as_str()).send()?;
    if res.status().is_success() {
        return Ok(res.text()?);
    }
    Err(ErrorKind::Http(res.status().as_u16()).into())
}

pub fn worker(
    id: u32,
    wait: Duration,
    client: Arc<Client>,
    sender: Sender<Task>,
    receiver: Receiver<Task>,
    results: Sender<Value>,
) {
    //let now = Instant::now();
    info!("worker#{}: starting vm", id);
    let (elapsed, vm) = measure(|| VM::new(sender).unwrap());
    info!("worker#{} vm started in {:?}", id, elapsed);

    loop {
        let timeout = after(wait);

        let task = select! {
            recv(receiver) -> ret => match ret {
                Ok(task) => task,
                Err(_) => break,
            },
            recv(timeout) -> _ => {
                info!("worker{} received timeout", id);
                break
            }
        };

        info!("worker#{}: received task: {}", id, task.id());

        let (elapsed, html) = measure(|| download(&client, id, &task));

        let html = match html {
            Err(e) => {
                info!("worker#{}: task#{}: got http error: {}", id, task.id(), e);
                continue;
            }
            Ok(html) => html,
        };

        info!(
            "worker#{}: task#{}: downloaded {:?} in {:?}",
            id,
            task.id(),
            task.url(),
            elapsed
        );

        let result = match vm.run2(&task, &html) {
            Err(e) => {
                error!("worker#{}: task#{} error {}", id, task.id(), e);
                continue;
            }
            Ok(r) => match r.get::<Value>() {
                Err(e) => {
                    error!("worker#{}: task#{} decode error {}", id, task.id(), e);
                    continue;
                }
                Ok(r) => r,
            },
        };

        if results.send(result).is_err() {
            info!("worker#{} results channel closed", id);
            return;
        }

        info!("worker#{} task done", id);
    }
}

impl Crawler {
    pub fn new() -> Crawler {
        CrawlerConfig::default().build()
    }

    pub fn add(&mut self, task: Task) {
        self.queue.push(task);
    }

    pub fn start(&mut self) -> usize {
        if self.running {
            return 0;
        }
        if self.queue.len() == 0 {
            return 0;
        }
        self.running = true;

        let (task_s, task_r) = unbounded::<Task>();

        let (result_s, result_r) = unbounded::<Value>();

        let results_path = match &self.cfg.output {
            Some(p) => p.clone(),
            None => "".to_owned(),
        };

        let count = Arc::new(Mutex::new(0));

        let cloned_count = count.clone();

        crossbeam::scope(move |scope| {
            let client = Arc::new(Client::new());

            for i in 0..self.cfg.workers {
                let (s, r) = (task_s.clone(), task_r.clone());
                let id = i + 1;
                let client = client.clone();
                let ret = result_s.clone();
                let timeout = self.cfg.timeout;
                scope.spawn(move || worker(id, timeout, client, s, r, ret));
            }

            for task in &self.queue {
                task_s.send(task.clone()).unwrap();
            }

            scope.spawn(move || {
                let ctx = Context::new().unwrap();

                let mut builder = duktape_modules::Builder::new();
                duktape_stdlib::register(&ctx, &mut builder, duktape_stdlib::Modules::all());
                duktape_modules::register(&ctx, builder);
                duktape_stdlib::init_runtime(&ctx);

                let module: Object = ctx.eval_main(results_path).unwrap();
                loop {
                    let result = match result_r.recv() {
                        Ok(r) => r,
                        Err(e) => break,
                    };

                    module.call::<_, _, ()>("exports", result);
                    let mut count = cloned_count.lock().unwrap();
                    *count += 1;
                }
            });
        });

        let c = count.lock().unwrap();
        c.clone()
    }
}
