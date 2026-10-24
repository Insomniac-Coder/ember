// reduce.rs: reduce.c's workload in safe Rust with scoped threads (std::thread::scope).
//   default       each thread returns its count; the counts are summed after the joins (Rust's idiom)
//   --cfg atomic  one AtomicU64, fetch_add(1, Relaxed) per word (phase 1's a15_workaround.rs spelling)
//   --cfg mutex   one Mutex<u64>, lock().unwrap() per word (Python's `with lock: total += 1`)
// Build: rustc -C opt-level=2 --edition 2021 -A unexpected_cfgs [--cfg atomic|mutex] reduce.rs
#[path = "../../ownership/bench/harness.rs"]
mod harness;
#[allow(unused_imports)]
use std::sync::{atomic::{AtomicU64, Ordering}, Mutex};

const TEXT_BYTES: usize = 64 << 20;
const TASKS: usize = 8;
const REPEAT: usize = 4;

fn main() {
    let _ = harness::start();
    let mut rng = harness::Rng::new(7);
    let mut text = vec![0u8; TEXT_BYTES];
    for b in text.iter_mut() { *b = if rng.next() % 6 == 0 { b' ' } else { b'a' + (rng.next() % 26) as u8 }; }
    for k in 1..=TASKS { text[k * (TEXT_BYTES / TASKS) - 1] = b' '; }
    let t0 = harness::start();
    let parts: Vec<&[u8]> = text.chunks(TEXT_BYTES / TASKS).collect();
    #[cfg(atomic)]
    let total = { let total = AtomicU64::new(0);
        std::thread::scope(|s| { for p in &parts { let total = &total; s.spawn(move || {
            for _ in 0..REPEAT { for w in p.windows(2) { if w[0] != b' ' && w[1] == b' ' { total.fetch_add(1, Ordering::Relaxed); } } }
        }); } });
        total.into_inner() };
    #[cfg(mutex)]
    let total = { let total = Mutex::new(0u64);
        std::thread::scope(|s| { for p in &parts { let total = &total; s.spawn(move || {
            for _ in 0..REPEAT { for w in p.windows(2) { if w[0] != b' ' && w[1] == b' ' { *total.lock().unwrap() += 1; } } }
        }); } });
        total.into_inner().unwrap() };
    #[cfg(not(any(atomic, mutex)))]
    let total: u64 = std::thread::scope(|s| {
        let hs: Vec<_> = parts.iter().map(|p| s.spawn(move || {
            let mut n = 0u64;
            for _ in 0..REPEAT { for w in p.windows(2) { if w[0] != b' ' && w[1] == b' ' { n += 1; } } }
            n
        })).collect();
        hs.into_iter().map(|h| h.join().unwrap()).sum()
    });
    let name = if cfg!(atomic) { "reduce-rust-atomic" } else if cfg!(mutex) { "reduce-rust-mutex" } else { "reduce-rust-joinsum" };
    harness::report(name, t0, total);
}
