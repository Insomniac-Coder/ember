// rust_cb_once.rs: workload cb_once in Rust. The idiom for a one-shot C callback with state: box the
// closure, give C `Box::into_raw` as the user data, and in the `extern "C"` trampoline take it back
// with `Box::from_raw` (unsafe) and call it: one allocation and one free per request. Nothing checks a
// second call (it would free the box twice). `TOTAL` is a static the main thread changes; the relaxed
// load and store compile to plain moves (no atomic add), as the model's module variable.
// The C library is written here in Rust, not inlined, with the same queue.
// Build: rustc -C opt-level=2 --edition 2021 rust_cb_once.rs
mod harness;
use std::os::raw::c_void;
use std::sync::atomic::{AtomicI64, Ordering};

const BATCH: usize = 1024;
const BATCHES: usize = 48828;

type Cb = extern "C" fn(*mut c_void);
struct Lib { queue: Vec<(Cb, *mut c_void)> }
impl Lib {
    #[inline(never)]
    fn request(&mut self, cb: Cb, ud: *mut c_void) { self.queue.push((cb, ud)); }
    #[inline(never)]
    fn pump(&mut self) {
        for &(cb, ud) in &self.queue { cb(ud); }
        self.queue.clear();
    }
}

static TOTAL: AtomicI64 = AtomicI64::new(0);

extern "C" fn trampoline<F: FnOnce()>(ud: *mut c_void) {
    let f = unsafe { Box::from_raw(ud as *mut F) };
    f();
}

fn request<F: FnOnce()>(lib: &mut Lib, f: F) {
    let ud = Box::into_raw(Box::new(f)) as *mut c_void;
    lib.request(std::hint::black_box(trampoline::<F> as Cb), ud);
}

fn main() {
    let t0 = harness::start();
    let mut lib = Lib { queue: Vec::with_capacity(BATCH) };
    let mut next_id: i64 = 0;
    for _ in 0..BATCHES {
        for _ in 0..BATCH {
            let id = next_id;
            request(&mut lib, move || { TOTAL.store(TOTAL.load(Ordering::Relaxed) + id, Ordering::Relaxed); });
            next_id += 1;
        }
        lib.pump();
    }
    harness::report("rust_cb_once", t0, (TOTAL.load(Ordering::Relaxed) as u64).wrapping_add(next_id as u64));
}
