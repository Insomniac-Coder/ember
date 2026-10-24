// harness.rs: the Rust twin of target/redesign/bench/harness.h. include!() it.
// Prints "<name> ms=<wall ms> peak_kb=<peak working set KB> checksum=<u64>", same LCG as h_rand().
#[repr(C)]
#[allow(non_snake_case)]
struct ProcessMemoryCounters {
    cb: u32, PageFaultCount: u32, PeakWorkingSetSize: usize, WorkingSetSize: usize,
    QuotaPeakPagedPoolUsage: usize, QuotaPagedPoolUsage: usize, QuotaPeakNonPagedPoolUsage: usize,
    QuotaNonPagedPoolUsage: usize, PagefileUsage: usize, PeakPagefileUsage: usize,
}
#[link(name = "kernel32")]
extern "system" {
    fn SetErrorMode(mode: u32) -> u32;
    fn GetCurrentProcess() -> *mut u8;
    fn K32GetProcessMemoryInfo(p: *mut u8, c: *mut ProcessMemoryCounters, cb: u32) -> i32;
}
struct H { t0: std::time::Instant, rng: u64 }
impl H {
    fn start() -> H {
        unsafe { SetErrorMode(0x0001 | 0x0002 | 0x8000); } // no error dialogs on the owner's screen
        H { t0: std::time::Instant::now(), rng: 1 }
    }
    fn seed(&mut self, s: u64) { self.rng = s; }
    #[inline(always)]
    fn rand(&mut self) -> u32 {
        self.rng = self.rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.rng >> 33) as u32
    }
    fn report(&self, name: &str, checksum: u64) {
        let ms = self.t0.elapsed().as_secs_f64() * 1000.0;
        let mut c = ProcessMemoryCounters { cb: std::mem::size_of::<ProcessMemoryCounters>() as u32,
            PageFaultCount: 0, PeakWorkingSetSize: 0, WorkingSetSize: 0, QuotaPeakPagedPoolUsage: 0,
            QuotaPagedPoolUsage: 0, QuotaPeakNonPagedPoolUsage: 0, QuotaNonPagedPoolUsage: 0,
            PagefileUsage: 0, PeakPagefileUsage: 0 };
        unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut c, c.cb); }
        println!("{} ms={:.3} peak_kb={} checksum={}", name, ms, c.PeakWorkingSetSize / 1024, checksum);
    }
}
