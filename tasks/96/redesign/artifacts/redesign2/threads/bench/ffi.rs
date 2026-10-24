// ffi.rs: ffi.c's workload with phase 1's Rust workaround (a20_ffi.rs): the user data is a raw
// `*mut Game`, turned back into `&mut Game` inside an `extern "C" fn`. The "library" is a function
// that only sees the function pointer and the pointer (black_box hides which function it is).
// Build: rustc -C opt-level=2 --edition 2021 ffi.rs
#[path = "../../ownership/bench/harness.rs"]
mod harness;
use std::os::raw::c_void;

const CALLS: i32 = 200_000_000;
pub struct Game { pub score: i64 }

#[inline(never)]
extern "C" fn fire(cb: extern "C" fn(*mut c_void), ud: *mut c_void, n: i32) {
    for _ in 0..n { cb(ud); }
}

extern "C" fn on_event(ud: *mut c_void) {
    let g: &mut Game = unsafe { &mut *(ud as *mut Game) };
    g.score += 1;
}

fn main() {
    let t0 = harness::start();
    let mut game = Box::new(Game { score: 0 });
    let cb = std::hint::black_box(on_event as extern "C" fn(*mut c_void));
    let ud = std::hint::black_box(&mut *game as *mut Game as *mut c_void);
    fire(cb, ud, CALLS);
    harness::report("ffi-rust-rawptr", t0, game.score as u64);
}
