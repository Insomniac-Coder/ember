// weakarr.rs: weakarr.c's workload in safe Rust, as Rust makes it be written: the subject holds
// Vec<Weak<Obs>>; the observers are owned by `all` (Rc). The same operations in the same order (the
// same random source) and the same checksum. A death drops the observer's last Rc; its Weak in the
// list then upgrades to None, and the program removes it at k, as the C programs do.
#[path = "../../graphs-callbacks/bench/harness.rs"]
mod harness;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

const NOBS: i64 = 1000;
const OPS: i64 = 300_000;

struct Obs { id: i64 }

fn main() {
    let t0 = harness::start();
    let mut rng = harness::Rng::new(7);
    let mut all: HashMap<i64, Rc<Obs>> = HashMap::new();   // the owners, by id
    let mut observers: Vec<Weak<Obs>> = Vec::new();
    for i in 0..NOBS {
        let o = Rc::new(Obs { id: i });
        observers.push(Rc::downgrade(&o));
        all.insert(i, o);
    }
    for op in 0..OPS {
        let k = (rng.next() as usize) % observers.len();
        if op % 100 == 0 {
            let id = observers[k].upgrade().expect("alive").id;
            all.remove(&id);                                  // the observer dies
            observers.remove(k);                              // the program removes the dead entry
            let o = Rc::new(Obs { id: NOBS + op });
            observers.push(Rc::downgrade(&o));
            all.insert(NOBS + op, o);
        } else {
            let w = observers.remove(k);
            observers.push(w);
        }
    }
    let mut sum: u64 = 0;
    for (k, w) in observers.iter().enumerate() {
        sum = sum.wrapping_add((w.upgrade().expect("alive").id * (k as i64 + 1)) as u64);
    }
    harness::report("weakarr-rust", t0, sum);
}
