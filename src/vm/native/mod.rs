use super::chunk::value::Val;
use std::time::{self, UNIX_EPOCH};

pub fn clock_native(_: usize, _: Vec<Val>) -> Val {
    let time = time::SystemTime::now();
    let value = time
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as f64;

    Val::number(value)
}
