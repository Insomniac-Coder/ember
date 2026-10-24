// lock.rs: lock.c's workload in safe Rust: Mutex<Vec<String>>, 8 scoped threads.
//   default        `let name = scores.lock().unwrap()[0].clone();` then use it (the E0515 workaround)
//   --cfg inplace  use `&g[0]` while the guard is held, then drop the guard
// Build: rustc -C opt-level=2 --edition 2021 -A unexpected_cfgs [--cfg inplace] lock.rs
#[path = "../../ownership/bench/harness.rs"]
mod harness;
use std::sync::Mutex;

const THREADS: u64 = 8;
const CALLS: u32 = 2_000_000;
const NAMES: u64 = 64;

fn make_name(k: u64) -> String { format!("player-{}", k.wrapping_mul(2654435761)) }
#[inline]
fn use_name(s: &[u8]) -> u64 {
    let mut h: u64 = 1469598103934665603;
    for _ in 0..4 { for &b in s { h = (h ^ b as u64).wrapping_mul(1099511628211); } }
    h
}

fn main() {
    let t0 = harness::start();
    let scores = Mutex::new((0..NAMES).map(make_name).collect::<Vec<String>>());
    std::thread::scope(|s| {
        for id in 0..THREADS {
            let scores = &scores;
            s.spawn(move || {
                let (mut acc, mut next) = (0u64, 1_000_000 * (id + 1));
                for c in 0..CALLS {
                    if id == 0 && c & 1023 == 0 {
                        let fresh = make_name(next); next += 1;
                        scores.lock().unwrap()[0] = fresh;
                    }
                    #[cfg(not(inplace))]
                    { let name = scores.lock().unwrap()[0].clone(); acc = acc.wrapping_add(use_name(name.as_bytes())); }
                    #[cfg(inplace)]
                    { let g = scores.lock().unwrap(); acc = acc.wrapping_add(use_name(g[0].as_bytes())); }
                }
                std::hint::black_box(acc);
            });
        }
    });
    let sum = use_name(scores.lock().unwrap()[0].as_bytes());
    harness::report(if cfg!(inplace) { "lock-rust-inplace" } else { "lock-rust-clone" }, t0, sum);
}
