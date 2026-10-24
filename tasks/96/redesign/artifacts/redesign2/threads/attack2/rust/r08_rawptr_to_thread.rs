// Twin of x09: a wrapper holding a raw pointer moved to another thread.
use std::thread;
pub struct Window { raw: *mut u8 }
impl Window { pub fn swap(&self) { let _ = self.raw; } }
pub fn go(w: Window) {
    let h = thread::spawn(move || w.swap());
    h.join().unwrap();
}
