// Twin of y14/x05: the parent holds the lock, two scoped tasks write through the guard.
use std::sync::Mutex;
use std::thread;
pub fn fill(m: &Mutex<Vec<i32>>) {
    let mut g = m.lock().unwrap();
    let v: &mut Vec<i32> = &mut *g;
    thread::scope(|s| {
        s.spawn(|| v.push(1));
        s.spawn(|| v.push(2));
    });
}
