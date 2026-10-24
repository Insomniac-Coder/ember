// rharness.rs: the Rust twin of target/redesign/bench/harness.h, included with include!().
// Same random generator, same output line: "<name> ms=<wall ms> peak_kb=<peak working set KB> checksum=<u64>".
// The only unsafe code is the two Windows calls the C harness also makes (no error dialogs; peak memory).
#[allow(dead_code)]
mod h {
    use std::time::Instant;
    #[repr(C)]
    struct Pmc { cb: u32, page_fault_count: u32, peak_working_set_size: usize, working_set_size: usize,
                 quota_peak_paged_pool_usage: usize, quota_paged_pool_usage: usize,
                 quota_peak_non_paged_pool_usage: usize, quota_non_paged_pool_usage: usize,
                 pagefile_usage: usize, peak_pagefile_usage: usize }
    #[link(name = "kernel32")]
    extern "system" {
        fn SetErrorMode(mode: u32) -> u32;
        fn GetCurrentProcess() -> isize;
        fn K32GetProcessMemoryInfo(p: isize, c: *mut Pmc, cb: u32) -> i32;
    }
    static mut RNG: u64 = 1;
    pub fn seed(s: u64) { unsafe { RNG = s; } }
    #[inline(always)]
    pub fn rand() -> u32 {
        unsafe {
            RNG = RNG.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (RNG >> 33) as u32
        }
    }
    pub fn start() -> Instant {
        unsafe { SetErrorMode(0x0001 | 0x0002 | 0x8000); }
        Instant::now()
    }
    pub fn report(name: &str, t0: Instant, checksum: u64) {
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        let mut p = Pmc { cb: std::mem::size_of::<Pmc>() as u32, page_fault_count: 0, peak_working_set_size: 0,
            working_set_size: 0, quota_peak_paged_pool_usage: 0, quota_paged_pool_usage: 0,
            quota_peak_non_paged_pool_usage: 0, quota_non_paged_pool_usage: 0, pagefile_usage: 0, peak_pagefile_usage: 0 };
        unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut p, p.cb); }
        println!("{} ms={:.3} peak_kb={} checksum={}", name, ms, p.peak_working_set_size / 1024, checksum);
    }
}
