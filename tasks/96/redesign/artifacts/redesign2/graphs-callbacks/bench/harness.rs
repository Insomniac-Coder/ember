// harness.rs (copied from target/redesign2/ownership/bench/harness.rs): the Rust twin of target/redesign/bench/harness.h (same LCG, same output line).
// The only unsafe code in the Rust programs is here (three Windows calls); the workloads are safe Rust.
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

#[repr(C)]
struct Pmc { cb: u32, faults: u32, peak_ws: usize, ws: usize, a: usize, b: usize, c: usize, d: usize, e: usize, f: usize }
#[link(name = "kernel32")]
extern "system" {
    fn SetErrorMode(mode: u32) -> u32;
    fn GetCurrentProcess() -> isize;
    fn K32GetProcessMemoryInfo(p: isize, pmc: *mut Pmc, cb: u32) -> i32;
}

pub fn start() -> Instant {
    unsafe { SetErrorMode(0x0001 | 0x0002 | 0x8000); }
    Instant::now()
}

pub fn report(name: &str, t0: Instant, checksum: u64) {
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    let mut p = Pmc { cb: std::mem::size_of::<Pmc>() as u32, faults: 0, peak_ws: 0, ws: 0, a: 0, b: 0, c: 0, d: 0, e: 0, f: 0 };
    unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut p, p.cb); }
    println!("{} ms={:.3} peak_kb={} checksum={}", name, ms, p.peak_ws / 1024, checksum);
}
