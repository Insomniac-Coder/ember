// W4 with a log callback in safe Rust: graphs-callbacks/bench/w4/w4_rust.rs with the log call added.
// The log is a Box<dyn FnMut(i64)> field of the dispatcher; a handler gets &mut Dispatcher and calls
// it (it is a field, so no take-and-put-back is needed for it). `logged` is Rc<Cell<i64>> because
// main reads it at the end. One of two identical closures is chosen at run time.
#[path = "../../graphs-callbacks/bench/harness.rs"]
mod harness;
use std::cell::Cell;
use std::rc::Rc;

const H: i64 = 10_000;
const E: i64 = 10_000_000;

type Handler = Box<dyn FnMut(&mut Dispatcher, i64, i64) -> i64>;

struct Dispatcher {
    handlers: Vec<Option<Handler>>,
    current: usize,
    log: Box<dyn FnMut(i64)>,
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
            (d.log)(k);
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
    let logged = Rc::new(Cell::new(0i64));
    let l2 = Rc::clone(&logged);
    let log: Box<dyn FnMut(i64)> = if std::env::args().count() > 100 {
        Box::new(move |k: i64| l2.set(l2.get() + (k & 1)))
    } else {
        Box::new(move |k: i64| l2.set(l2.get() + (k & 1)))
    };
    let mut d = Dispatcher { handlers: Vec::new(), current: 0, log };
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
    harness::report("w4log-rust", t0, (total.get() + s + logged.get()) as u64);
}
