// rharness.rs: the Rust twin of target/redesign/bench/harness.h. Same LCG, same output line:
// "<name> ms=<wall ms> peak_kb=<peak working set KB> checksum=<u64>".
// The ONLY unsafe code in the Rust baselines is the one K32GetProcessMemoryInfo call in report(),
// which reads the peak working set. The workloads themselves are safe Rust.
// (No SetErrorMode here: safe Rust ends a failure with a panic message and exit code 101, no box.)
#![allow(dead_code)]
use std::time::Instant;

pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Rng { Rng(seed) }
    #[inline(always)]
    pub fn next(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }
}

// PROCESS_MEMORY_COUNTERS: cb, PageFaultCount, then eight SIZE_T fields.
#[repr(C)]
struct Pmc { cb: u32, faults: u32, peak_ws: usize, rest: [usize; 7] }
#[link(name = "kernel32")]
extern "system" {
    fn K32GetProcessMemoryInfo(process: isize, pmc: *mut Pmc, cb: u32) -> i32;
}

pub fn start() -> Instant { Instant::now() }

pub fn report(name: &str, t0: Instant, checksum: u64) {
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    let mut p = Pmc { cb: std::mem::size_of::<Pmc>() as u32, faults: 0, peak_ws: 0, rest: [0; 7] };
    // -1 is the current-process pseudo-handle (what GetCurrentProcess returns).
    let ok = unsafe { K32GetProcessMemoryInfo(-1, &mut p, p.cb) };
    let kb = if ok != 0 { p.peak_ws / 1024 } else { 0 };
    println!("{} ms={:.3} peak_kb={} checksum={}", name, ms, kb, checksum);
}
