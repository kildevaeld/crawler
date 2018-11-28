use super::task::{ParseTask, Task};
use super::utils::measure;
use super::vm::VM;
use crossbeam;
use duktape::prelude::*;
use duktape::Context;
use duktape_cjs::{self, CJSContext};
use duktape_stdlib;
use reqwest::Client;
use std::time::{Duration, Instant};
use value::Value;

use crossbeam_channel::{after, bounded, unbounded, Receiver, Sender};

pub struct CrawlerConfig {
    producers: u32,
    consumers: u32,
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
            output: None,
        }
    }
}

pub struct Crawler {
    queue: Vec<Task>,
    running: bool,
    cfg: CrawlerConfig,
}

fn producer(id: u32, sender: Sender<ParseTask>, receiver: Receiver<Task>) {
    let client = Client::new();

    loop {
        let task = match receiver.recv() {
            Ok(task) => task,
            Err(_) => break,
        };

        info!(
            "producer#{}: received task#{}: {}",
            id,
            task.id(),
            task.url()
        );

        let (elapsed, res) = measure(|| client.get(task.url().as_str()).send());

        let mut res = match res {
            Ok(res) => {
                info!(
                    "producer#{}: task#{}: responsed in {:?}",
                    id,
                    task.id(),
                    elapsed
                );

                res
            }
            Err(e) => {
                info!("producer#{}: task#{}: got http error: {}", id, task.id(), e);
                continue;
            }
        };

        let (elapsed, res) = measure(|| res.text());

        let text = match res {
            Ok(text) => {
                info!(
                    "producer#{}: task#{}: downloaded in {:?}",
                    id,
                    task.id(),
                    elapsed
                );
                text
            }
            Err(e) => {
                info!(
                    "producer#{}: task#{}: got http response error: {}",
                    id,
                    task.id(),
                    e
                );
                continue;
            }
        };

        let task_id = task.id().clone();

        match sender.send(task.into_parse_task(&text)) {
            Err(e) => {
                info!("producer#{}: sender closed: {}", id, e);
                break;
            }
            _ => {
                info!("producer#{}: sent task#{}", id, task_id);
            }
        }
    }
    info!("producer#{}: done", id);
}

fn consumer(
    id: u32,
    wait: Duration,
    sender: Sender<Task>,
    receiver: Receiver<ParseTask>,
    results: Sender<Value>,
) {
    let now = Instant::now();
    info!("consumer#{}: starting vm", id);
    let vm = VM::new(sender).unwrap();
    let lastnow = Instant::now();
    info!(
        "consumer#{} vm started in {:?}",
        id,
        lastnow.duration_since(now)
    );

    loop {
        let timeout = after(wait);

        let task = select!{
            recv(receiver) -> ret => match ret {
                Ok(task) => task,
                Err(_) => break,
            },
            recv(timeout) -> _ => break
        };

        info!("consumer#{}: got task#{}", id, task.id);

        let result = match vm.run(&task) {
            Err(e) => {
                error!("consumer#{}: task#{} error {}", id, task.id, e);
                continue;
            }
            Ok(r) => match r.get::<Value>() {
                Err(e) => {
                    error!("consumer#{}: task#{} decode error {}", id, task.id, e);
                    continue;
                }
                Ok(r) => r,
            },
        };

        results.send(result);

        info!("consumer#{} task done", id);
    }
    info!("consumer#{} done", id);
}

impl Crawler {
    pub fn new() -> Crawler {
        CrawlerConfig::default().build()
    }

    pub fn add(&mut self, task: Task) {
        self.queue.push(task);
    }

    pub fn start(&mut self) {
        if self.running {
            return;
        }
        if self.queue.len() == 0 {
            return;
        }
        self.running = true;

        let (parse_task_s, parse_task_r) = bounded::<ParseTask>(self.cfg.buffer_size);
        let (task_s, task_r) = unbounded::<Task>();

        for task in &self.queue {
            task_s.send(task.clone()).unwrap();
        }

        let (result_s, result_r) = unbounded::<Value>();

        let results_path = match &self.cfg.output {
            Some(p) => p.clone(),
            None => "".to_owned(),
        };

        crossbeam::scope(move |scope| {
            for i in 0..self.cfg.producers {
                let (s, r) = (parse_task_s.clone(), task_r.clone());
                let id = i + 1;
                scope.spawn(move || producer(id, s, r));
            }

            for i in 0..self.cfg.consumers {
                let (s, r) = (task_s.clone(), parse_task_r.clone());
                let id = i + 1;
                let timeout = self.cfg.timeout;
                let ret = result_s.clone();
                scope.spawn(move || consumer(id, timeout, s, r, ret));
            }

            scope.spawn(move || {
                let ctx = Context::new().unwrap();

                let mut builder = duktape_cjs::RequireBuilder::new();
                duktape_stdlib::init(&ctx, &mut builder, duktape_stdlib::Modules::all());
                duktape_cjs::register(&ctx, builder);
                duktape_stdlib::init_runtime(&ctx);

                let module: Object = ctx.eval_main(results_path).unwrap();

                loop {
                    let result = match result_r.recv() {
                        Ok(r) => r,
                        Err(e) => break,
                    };

                    module.call::<_, _, ()>("exports", result);
                }
            });
        });
    }
}
