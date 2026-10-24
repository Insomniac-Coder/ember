// W4 in safe Rust, as Rust makes it be written. The handler list holds boxed closures; `total` is an
// Rc<Cell<i64>> because a stored closure may not borrow main's local (E0373, phase1-sharing.md A12).
// A handler gets `&mut Dispatcher`, so the dispatcher cannot call it in place (E0499, A32): it takes
// the closure out of its slot (leaving None), calls it, and puts it back if the slot is still the
// empty one; a handler that removed itself is dropped after it returns. This is the same mechanism
// this report proposes Ember emit, written by hand.
#[path = "../harness.rs"]
mod harness;
use std::cell::Cell;
use std::rc::Rc;

const H: i64 = 10_000;
const E: i64 = 10_000_000;

type Handler = Box<dyn FnMut(&mut Dispatcher, i64, i64) -> i64>;

struct Dispatcher {
    handlers: Vec<Option<Handler>>,
    current: usize,
}

impl Dispatcher {
    fn register(&mut self, h: Handler) {
        self.handlers.push(Some(h));
    }
    fn unregister_current(&mut self) {
        self.handlers.remove(self.current);
    }
    fn dispatch(&mut self, k: i64) {
        let i = (k as usize) % self.handlers.len();
        self.current = i;
        let mut h = self.handlers[i].take().expect("handler re-entered");
        h(self, k, k % 1000);
        if i < self.handlers.len() && self.handlers[i].is_none() {
            self.handlers[i] = Some(h);
        }
    }
}

fn make_handler(m: i64, total: Rc<Cell<i64>>) -> Handler {
    let mut counter: i64 = 0;
    Box::new(move |d: &mut Dispatcher, k: i64, v: i64| -> i64 {
        if v >= 0 {
            counter += v * m;
            total.set(total.get() + 1);
            if k % 100_000 == 99_999 {
                d.unregister_current();
                d.register(make_handler(k % 17 + 1, Rc::clone(&total)));
            }
        }
        counter
    })
}

fn main() {
    let t0 = harness::start();
    let total = Rc::new(Cell::new(0i64));
    let mut d = Dispatcher { handlers: Vec::new(), current: 0 };
    for i in 0..H {
        d.register(make_handler(i % 13 + 1, Rc::clone(&total)));
    }
    for k in 0..E {
        d.dispatch(k);
    }
    let mut s: i64 = 0;
    let mut hs = std::mem::take(&mut d.handlers);
    for h in hs.iter_mut() {
        s += (h.as_mut().unwrap())(&mut d, 0, -1);
    }
    harness::report("w4-rust", t0, (total.get() + s) as u64);
}
