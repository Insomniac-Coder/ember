// Rust: configure an object, then share it with a thread (Arc after the writes).
use std::sync::Arc;
struct Config { scale: i64 }
fn main() {
    let mut cfg = Config { scale: 2 };
    cfg.scale = 3;                       // a write after construction, before the share
    let cfg = Arc::new(cfg);
    let c2 = Arc::clone(&cfg);
    let h = std::thread::spawn(move || c2.scale);
    println!("{}", h.join().unwrap() + cfg.scale);
}
