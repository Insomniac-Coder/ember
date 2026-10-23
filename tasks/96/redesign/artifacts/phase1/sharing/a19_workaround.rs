// A19 workaround: an atomic and a Mutex (Mutex::new is const since Rust 1.63).
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
static FRAME: AtomicU64 = AtomicU64::new(0);
static LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub fn tick() {
    let n = FRAME.fetch_add(1, Ordering::Relaxed) + 1;
    LOG.lock().unwrap().push(format!("tick {}", n));
}
