extern crate crawler;
extern crate env_logger;
#[macro_use]
extern crate clap;

use std::fs;

fn main() {
    env_logger::init();

    let matches = clap_app!(app =>
        (version: "10.0")
        (author: "Rasmus")
        (@arg path: * "path")
    )
    .get_matches();

    let mut cfg = crawler::CrawlerConfig::default();
    cfg.output("example/index.js").consumers(2).producers(2);

    let mut c = cfg.build();

    let iter = fs::read_dir(matches.value_of("path").unwrap()).unwrap();

    for path in iter {
        let path = path.unwrap();
        if !path.path().is_dir() {
            continue;
        }
        let pack = crawler::Package::load(path.path()).unwrap();
        c.add(pack.task().unwrap());
    }

    // let task = crawler::Task::new(
    //     "http://loppen.dk",
    //     crawler::Work::path("example/loppen/index.js"),
    // )
    // .unwrap();

    //let pack = crawler::Package::load("./example/loppen").unwrap();
    // let pack = crawler::Package::load("./example/amagerbio").unwrap();
    // println!("{:?}", pack);
    // let mut cfg = crawler::CrawlerConfig::default();
    // cfg.output("example/index.js");

    // let mut c = cfg.build();
    // c.add(pack.task().unwrap());

    c.start();

    // println!("{:?}", task);

    // let queue = crawler::createTaskQueue();

    // queue.borrow_mut().push(task);

    // let vm = crawler::VM::new("example/loppen/index.js").unwrap();

    // vm.run(
    //     queue,
    //     "<div><span class=\"inner\">Hello, Rapper</span></div>",
    // )
    // .unwrap();

    println!("Hello, world!");
}
