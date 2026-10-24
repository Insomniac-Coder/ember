// share_cell.rs: the same workload in tuned safe Rust: Rc<Player> with a Cell for the plain field and a
// RefCell only around the list, so a plain-field write costs no borrow flag.
mod harness;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

const P: usize = 1 << 18;
const W: usize = 1 << 20;
const FRAMES: usize = 32;

struct Player { hp: Cell<i32>, x: i32, inv: RefCell<Vec<i32>> }

fn main() {
    let t0 = harness::start();
    let mut rng = harness::Rng::new(42);
    let players: Vec<Rc<Player>> = (0..P)
        .map(|i| Rc::new(Player { hp: Cell::new(1000), x: (i & 1023) as i32, inv: RefCell::new(Vec::new()) }))
        .collect();
    let mut ws: Vec<Rc<Player>> = Vec::with_capacity(W);
    for _ in 0..W { ws.push(Rc::clone(&players[rng.next() as usize & (P - 1)])); }
    let mut acc: u64 = 0;
    for f in 0..FRAMES {
        for i in 0..W {
            {
                let t = &*ws[i];
                t.hp.set(t.hp.get().wrapping_sub(1));
                acc = acc.wrapping_add((t.x as u32 as u64).wrapping_mul(t.inv.borrow().len() as u64 + 1));
                if (i & 7) == (f & 7) {
                    let mut inv = t.inv.borrow_mut();
                    inv.push(f as i32);
                    if inv.len() > 8 {
                        for v in inv.iter() { acc = acc.wrapping_add(*v as u32 as u64); }
                        inv.clear();
                    }
                }
            }
            if rng.next() & 15 == 0 {
                let r = rng.next() as usize & (P - 1);
                ws[i] = Rc::clone(&players[r]);
            }
        }
    }
    let mut sum = acc;
    for p in players.iter() {
        sum = sum.wrapping_add(p.hp.get() as u32 as u64).wrapping_add(p.inv.borrow().len() as u64);
    }
    harness::report("share-rust-cell", t0, sum);
}
