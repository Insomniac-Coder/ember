// globals.rs: globals.c's workload in safe Rust.
//   default     thread_local! with Cell and RefCell (Rust's one-thread idiom for a mutable global)
//   --cfg sync  static AtomicU64 (Relaxed, phase 1's a19_workaround.rs) and static Mutex<Vec<i64>>
// Build: rustc -C opt-level=2 --edition 2021 -A unexpected_cfgs [--cfg sync] globals.rs
#[path = "../../ownership/bench/harness.rs"]
mod harness;
#[allow(unused_imports)]
use std::cell::{Cell, RefCell};
#[allow(unused_imports)]
use std::sync::{atomic::{AtomicU64, Ordering}, Mutex};

const CALLS: u32 = 100_000_000;

#[cfg(sync)]
static FRAME: AtomicU64 = AtomicU64::new(0);
#[cfg(sync)]
static LOG: Mutex<Vec<i64>> = Mutex::new(Vec::new());
#[cfg(not(sync))]
thread_local! {
    static FRAME: Cell<i64> = const { Cell::new(0) };
    static LOG: RefCell<Vec<i64>> = const { RefCell::new(Vec::new()) };
}

#[inline(never)]
fn tick(checksum: &mut u64) {
    #[cfg(sync)]
    {
        let n = FRAME.fetch_add(1, Ordering::Relaxed) as i64 + 1;
        let mut log = LOG.lock().unwrap();
        log.push(n);
        if log.len() == 4096 { for v in log.iter() { *checksum += *v as u64; } log.clear(); }
    }
    #[cfg(not(sync))]
    {
        let n = FRAME.with(|f| { f.set(f.get() + 1); f.get() });
        LOG.with(|l| { let mut log = l.borrow_mut(); log.push(n);
            if log.len() == 4096 { for v in log.iter() { *checksum += *v as u64; } log.clear(); } });
    }
}

fn main() {
    let t0 = harness::start();
    let mut checksum = 0u64;
    for _ in 0..CALLS { tick(&mut checksum); }
    harness::report(if cfg!(sync) { "globals-rust-atomic-mutex" } else { "globals-rust-threadlocal" }, t0, checksum);
}
