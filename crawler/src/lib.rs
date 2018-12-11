extern crate duktape;
extern crate duktape_cjs;
extern crate duktape_stdlib;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
#[macro_use]
extern crate log;

extern crate reqwest;
extern crate url;
extern crate url_serde;
#[macro_use]
extern crate error_chain;
extern crate crossbeam;
#[macro_use]
extern crate crossbeam_channel;
extern crate uuid;
extern crate value;

#[macro_use]
pub mod utils;
mod cheerio;
mod crawler;
pub mod error;
mod manifest;
mod queue;
mod task;

mod vm;

pub use self::crawler::*;
pub use self::manifest::*;
pub use self::queue::*;
pub use self::task::*;
pub use self::vm::VM;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
