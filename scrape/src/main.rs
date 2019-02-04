extern crate crawler;
extern crate env_logger;
#[macro_use]
extern crate clap;
extern crate spinners;

use std::fs;
use std::io;

fn load_packages(crawler: &mut crawler::Crawler, path: &str) -> crawler::error::Result<u32> {
    let iter = fs::read_dir(path)?;

    let mut count = 0;
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
        count += 1;
    }

    Ok(count)
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
    print!("Scanning for packages ... ");
    let count = load_packages(&mut c, matches.value_of("path").unwrap()).unwrap_or_else(|e| {
        panic!("could not");
        0
    });

    println!("\rFound {} packages       ", count);

    let (e, count) = if cfg!(debug_assertions) {
        crawler::utils::measure(|| c.start())
    } else {
        let spin = spinners::Spinner::new(
            spinners::Spinners::Dots12,
            "Executing tasks ...".to_string(),
        );
        let (e, c) = crawler::utils::measure(|| c.start());
        spin.stop();
        (e, c)
    };

    println!("\rExecuted {} tasks in {:?}", count, e);
}
