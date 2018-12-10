use std::time::{Duration, Instant};

pub fn measure<T, F: FnOnce() -> T>(f: F) -> (Duration, T) {
    let start = Instant::now();
    let r = f();
    let now = Instant::now();
    (now.duration_since(start), r)
}

use super::error::{ErrorKind, Result};
use std::path::{Path, PathBuf};

pub fn join<T: AsRef<Path>, S: AsRef<str>>(base: T, cmp: S) -> Result<PathBuf> {
    let mut path: String = cmp.as_ref().to_owned();
    let mut base: PathBuf = base.as_ref().to_path_buf();
    if path.starts_with("./") {
        path = path.trim_left_matches("./").to_string();
    }

    while path.starts_with("../") {
        base = match base.parent() {
            Some(parent) => parent.to_path_buf(),
            None => return Err("could not resolve".into()),
        };
        path = path.trim_left_matches("..").to_string();
    }

    Ok(base.join(path))
}

pub fn join_slice<T: AsRef<Path>, S: AsRef<str>>(base: T, cmps: &[S]) -> Result<PathBuf> {
    let mut path = base.as_ref().to_path_buf();

    for p in cmps {
        path = join(&path, p)?;
    }

    Ok(path)
}
