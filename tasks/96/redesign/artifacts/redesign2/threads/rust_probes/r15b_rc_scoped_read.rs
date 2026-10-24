// A15: scoped threads that only read through an Rc (Ember's class handle is an Rc).
use std::rc::Rc;
pub struct Config { pub scale: i32 }
pub fn run(cfg: Rc<Config>, xs: &[i32]) -> i32 {
    std::thread::scope(|s| {
        let a = s.spawn(|| xs[0] * cfg.scale);
        let b = s.spawn(|| xs[1] * cfg.scale);
        a.join().unwrap() + b.join().unwrap()
    })
}
