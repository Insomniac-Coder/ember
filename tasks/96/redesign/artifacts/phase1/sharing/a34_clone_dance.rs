// A34: two worker threads append to one log.
use std::sync::{Arc, Mutex};
use std::thread;
pub fn start(log: Arc<Mutex<Vec<String>>>) {
    let a = thread::spawn(move || log.lock().unwrap().push("a".to_string()));
    let b = thread::spawn(move || log.lock().unwrap().push("b".to_string()));
    a.join().unwrap();
    b.join().unwrap();
}
