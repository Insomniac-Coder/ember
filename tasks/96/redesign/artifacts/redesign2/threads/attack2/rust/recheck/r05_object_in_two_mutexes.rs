// Rust: one shared object reachable from two Mutexes cannot be written through either guard.
use std::sync::{Arc, Mutex};
struct Player { hp: i64 }
fn main() {
    let p = Arc::new(Player { hp: 10 });
    let a = Mutex::new(vec![Arc::clone(&p)]);
    let b = Mutex::new(vec![Arc::clone(&p)]);
    std::thread::scope(|s| {
        s.spawn(|| { let mut g = a.lock().unwrap(); g[0].hp -= 1; });
        s.spawn(|| { let mut g = b.lock().unwrap(); g[0].hp -= 1; });
    });
    println!("{}", p.hp);
}
