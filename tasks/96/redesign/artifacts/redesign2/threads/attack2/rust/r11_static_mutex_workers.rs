// Rust's spelling of y24: a static Mutex<Vec<i64>> appended to by worker threads.
use std::sync::Mutex;
use std::thread;
static RESULTS: Mutex<Vec<i64>> = Mutex::new(Vec::new());
pub fn go() -> usize {
    let hs: Vec<_> = (0..4).map(|i| thread::spawn(move || { RESULTS.lock().unwrap().push(i * i); })).collect();
    for h in hs { h.join().unwrap(); }
    RESULTS.lock().unwrap().len()
}
