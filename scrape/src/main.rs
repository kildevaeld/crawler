extern crate crawler;
extern crate env_logger;
#[macro_use]
extern crate clap;

use std::fs;
use std::io;

fn load_packages(crawler: &mut crawler::Crawler, path: &str) -> crawler::error::Result<()> {
    let iter = fs::read_dir(path)?;

    for path in iter {
        let path = path?;

        if !path.path().is_dir() {
            continue;
        }

        let pack = match crawler::Package::load(path.path()) {
            Some(p) => p,
            None => continue,
        };

        crawler.add(pack.task()?);
    }

    Ok(())
}

fn main() {
    env_logger::init();

    let matches = clap_app!(app =>
        (version: "10.0")
        (author: "Rasmus")
        (@arg path: * "path")
        (@arg workers: -w --workers <workers> "workers")
        (@arg script: -o --output <script> "script to send output")
    )
    .get_matches();

    let workers = value_t!(matches, "workers", u32).unwrap_or(4);

    let mut cfg = crawler::CrawlerConfig::default();

    if let Some(o) = matches.value_of("script") {
        cfg.output(o);
    }

    cfg.workers(workers);

    let mut c = cfg.build();

    load_packages(&mut c, matches.value_of("path").unwrap())
        .unwrap_or_else(|e| println!("could not {}", e));
    // let iter = fs::read_dir(matches.value_of("path").unwrap()).unwrap();

    // for path in iter {
    //     let path = path.unwrap();
    //     if !path.path().is_dir() {
    //         continue;
    //     }
    //     let pack = crawler::Package::load(path.path()).unwrap();
    //     c.add(pack.task().unwrap());
    // }

    let (e, count) = crawler::utils::measure(|| c.start());
    println!("executed {} tasks in {:?}", count, e);
}
