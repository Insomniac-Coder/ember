// counts.rs: counts.c's workload in safe Rust: `Rc` (plain counts), or `Arc` with `--cfg arc`.
// Build: rustc -C opt-level=2 --edition 2021 counts.rs -o counts_rc.exe   (add --cfg arc for Arc;
// --cfg steprc also clones and drops the handle on every step, as counts.c's -DSTEPRC)
#[path = "../../ownership/bench/harness.rs"]
mod harness;
use std::cell::{Cell, OnceCell};
#[cfg(not(arc))]
use std::rc::Rc as Counted;
#[cfg(arc)]
use std::sync::Arc as Counted;

const P_COUNT: usize = 1 << 18;
const W_COUNT: usize = 1 << 20;
const FRAMES: u32 = 32;

struct Obj { hp: Cell<i32>, x: i32, buddy: OnceCell<Counted<Obj>>, _pad: i64 }

fn main() {
    let t0 = harness::start();
    let mut rng = harness::Rng::new(42);
    let objs: Vec<Counted<Obj>> = (0..P_COUNT)
        .map(|i| Counted::new(Obj { hp: Cell::new(1000), x: (i & 1023) as i32, buddy: OnceCell::new(), _pad: 0 }))
        .collect();
    for i in 0..P_COUNT {
        let b = objs[((i as u32).wrapping_mul(2654435761).wrapping_add(1) as usize) & (P_COUNT - 1)].clone();
        let _ = objs[i].buddy.set(b);
    }
    let mut ws: Vec<Counted<Obj>> = (0..W_COUNT).map(|_| objs[rng.next() as usize & (P_COUNT - 1)].clone()).collect();
    let mut acc: u64 = 0;
    for _ in 0..FRAMES {
        for i in 0..W_COUNT {
            #[cfg(steprc)]
            let held = ws[i].clone();
            let t = &ws[i];
            t.hp.set(t.hp.get() - 1);
            acc = acc.wrapping_add((t.hp.get() ^ t.x) as u32 as u64);
            if rng.next() & 15 == 0 {
                ws[i] = objs[rng.next() as usize & (P_COUNT - 1)].clone();
            }
            #[cfg(steprc)]
            drop(held);
        }
    }
    let name = match (cfg!(arc), cfg!(steprc)) {
        (false, false) => "counts-rust-rc", (true, false) => "counts-rust-arc",
        (false, true) => "counts-rust-rc-s", (true, true) => "counts-rust-arc-s" };
    harness::report(name, t0, acc);
    std::mem::forget(ws); // teardown skipped, as in the C models
    std::mem::forget(objs);
}
