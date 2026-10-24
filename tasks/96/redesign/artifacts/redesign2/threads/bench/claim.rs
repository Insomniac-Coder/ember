// claim.rs: claim.c's workload in safe Rust, 8 scoped threads.
//   default       Vec<&mut Player> into one Vec<Player>, shuffled the same way, split with chunks_mut:
//                 the borrow checker proves no object is in two parts (Rust's zero-cost answer, values)
//   --cfg mutex   Vec<Arc<Mutex<Player>>>, shuffled, one lock per object per pass (Rust's answer for
//                 objects that are shared, as Ember's class objects are)
// Build: rustc -C opt-level=2 --edition 2021 -A unexpected_cfgs [--cfg mutex] claim.rs
#[path = "../../ownership/bench/harness.rs"]
mod harness;
#[allow(unused_imports)]
use std::sync::{Arc, Mutex};

const N: usize = 1 << 20;
const TASKS: usize = 8;
const PASSES: usize = 64;

#[allow(dead_code)]
pub struct Player { hp: i32, x: i32, more: [i64; 2] }

fn shuffle<T>(v: &mut Vec<T>, rng: &mut harness::Rng) {
    for i in (1..N).rev() { let j = rng.next() as usize % (i + 1); v.swap(i, j); }
}

fn main() {
    let _ = harness::start();
    let mut rng = harness::Rng::new(11);
    #[cfg(not(mutex))]
    let sum = {
        let mut store: Vec<Player> = (0..N).map(|i| Player { hp: 1_000_000, x: i as i32, more: [0; 2] }).collect();
        let t0;
        {
            let mut refs: Vec<&mut Player> = store.iter_mut().collect();
            shuffle(&mut refs, &mut rng);
            t0 = harness::start();
            for _ in 0..PASSES {
                std::thread::scope(|s| {
                    for part in refs.chunks_mut(N / TASKS) {
                        s.spawn(move || for p in part.iter_mut() { p.hp -= 1; p.x += p.hp & 1; });
                    }
                });
            }
        }
        (store.iter().map(|p| (p.hp ^ p.x) as u32 as u64).sum::<u64>(), t0)
    };
    #[cfg(mutex)]
    let sum = {
        let store: Vec<Arc<Mutex<Player>>> = (0..N).map(|i| Arc::new(Mutex::new(Player { hp: 1_000_000, x: i as i32, more: [0; 2] }))).collect();
        let mut handles: Vec<Arc<Mutex<Player>>> = store.iter().cloned().collect();
        shuffle(&mut handles, &mut rng);
        let t0 = harness::start();
        for _ in 0..PASSES {
            std::thread::scope(|s| {
                for part in handles.chunks(N / TASKS) {
                    s.spawn(move || for h in part { let mut p = h.lock().unwrap(); p.hp -= 1; p.x += p.hp & 1; });
                }
            });
        }
        (store.iter().map(|h| { let p = h.lock().unwrap(); (p.hp ^ p.x) as u32 as u64 }).sum::<u64>(), t0)
    };
    harness::report(if cfg!(mutex) { "claim-rust-arc-mutex" } else { "claim-rust-refs-chunks" }, sum.1, sum.0);
}
