// Rust: a helper that changes the protected data, called with the lock held.
use std::sync::Mutex;
struct Inner { words: i64 }
fn bump(s: &mut Inner) { s.words += 1; }
fn main() {
    let m = Mutex::new(Inner { words: 0 });
    std::thread::scope(|sc| {
        for _ in 0..8 { sc.spawn(|| { let mut g = m.lock().unwrap(); bump(&mut g); }); }
    });
    println!("{}", m.lock().unwrap().words);
}
