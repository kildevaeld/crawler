use std::time::{Duration, Instant};

pub fn measure<T, F: FnOnce() -> T>(f: F) -> (Duration, T) {
    let start = Instant::now();
    let r = f();
    let now = Instant::now();
    (now.duration_since(start), r)
}
