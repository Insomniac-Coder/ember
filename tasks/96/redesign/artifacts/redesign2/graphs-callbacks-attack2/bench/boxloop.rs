// boxloop.rs: the boxloop.c workload in safe Rust, as Rust makes it be written: the stored closure
// cannot borrow main's local (E0373), so `total` is an Rc<Cell<i64>> shared with the closure.
#[path = "../../graphs-callbacks/bench/harness.rs"]
mod harness;
use std::cell::Cell;
use std::rc::Rc;

const N: i64 = 65536;
const REPS: i32 = 2000;

fn main() {
    let t0 = harness::start();
    let xs: Vec<i64> = (0..N).map(|i| i % 1000).collect();
    let total = Rc::new(Cell::new(0i64));
    let mut callbacks: Vec<Box<dyn Fn() -> i64>> = Vec::new();
    let t = Rc::clone(&total);
    callbacks.push(Box::new(move || t.get()));
    for _ in 0..REPS {
        for x in xs.iter() {
            total.set(total.get() + x);
        }
    }
    let r = callbacks[0]();
    harness::report("boxloop-rust-rc-cell", t0, r as u64);
}
