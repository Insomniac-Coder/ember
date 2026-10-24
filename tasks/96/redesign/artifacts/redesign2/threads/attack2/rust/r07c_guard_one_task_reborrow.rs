// One scoped task writes through `&mut *g` while the parent holds the guard: accepted, and race-free.
use std::sync::Mutex;
use std::thread;
pub fn fill(m: &Mutex<Vec<i32>>) {
    let mut g = m.lock().unwrap();
    let v: &mut Vec<i32> = &mut *g;
    thread::scope(|s| {
        s.spawn(|| v.push(1));
    });
}
