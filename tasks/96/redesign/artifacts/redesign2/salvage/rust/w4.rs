// Safe Rust, W4 (callbacks with captured mutable state). Each handler is a boxed closure that owns
// its counter and multiplier and shares `total` through Rc<Cell<i64>>. Rust refuses a handler that
// changes the dispatcher's list from inside its own call (the list is borrowed for the call), so the
// handler RETURNS a request and the dispatcher unregisters it and appends the replacement right
// after the call: the same observable result, in the order Rust allows. A call with v < 0 asks the
// handler for its counter (the same device as the D3 model), for the checksum.
#[path = "harness.rs"]
mod harness;
use std::cell::Cell;
use std::rc::Rc;

const H: i64 = 10_000;
const E: i64 = 10_000_000;

type Handler = Box<dyn FnMut(i64, i64) -> (i64, bool)>;

fn make_handler(m: i64, total: Rc<Cell<i64>>) -> Handler {
    let mut counter: i64 = 0;
    Box::new(move |k, v| {
        if v >= 0 {
            counter += v * m;
            total.set(total.get() + 1);
            if k % 100_000 == 99_999 { return (counter, true); }
        }
        (counter, false)
    })
}

fn main() {
    let t0 = harness::start();
    let total = Rc::new(Cell::new(0i64));
    let mut hs: Vec<Handler> = Vec::new();
    for i in 0..H { hs.push(make_handler(i % 13 + 1, total.clone())); }
    for k in 0..E {
        let cur = (k % hs.len() as i64) as usize;
        let (_, replace) = (hs[cur])(k, k % 1000);
        if replace {
            hs.remove(cur);
            hs.push(make_handler(k % 17 + 1, total.clone()));
        }
    }
    let mut s = total.get();
    for h in hs.iter_mut() { s += h(0, -1).0; }
    harness::report("rust-w4", t0, s as u64);
}
