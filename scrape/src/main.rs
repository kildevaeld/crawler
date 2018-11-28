extern crate crawler;
extern crate env_logger;

fn main() {
    env_logger::init();
    // let task = crawler::Task::new(
    //     "http://loppen.dk",
    //     crawler::Work::path("example/loppen/index.js"),
    // )
    // .unwrap();

    let pack = crawler::Package::load("./example/loppen").unwrap();
    println!("{:?}", pack);
    let mut cfg = crawler::CrawlerConfig::default();
    cfg.output("example/index.js");

    let mut c = cfg.build();
    c.add(pack.task().unwrap());

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
