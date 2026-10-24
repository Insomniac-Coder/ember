// Twin of x05b: one scoped task captures the guard itself.
use std::sync::Mutex;
use std::thread;
pub fn fill(m: &Mutex<Vec<i32>>) {
    let mut g = m.lock().unwrap();
    thread::scope(|s| {
        s.spawn(|| g.push(1));
    });
}
