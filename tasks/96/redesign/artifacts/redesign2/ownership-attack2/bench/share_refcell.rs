// share_refcell.rs: the shared-object workload (share.c) in safe Rust, the way Rust's book shares a
// mutable object: Rc<RefCell<Player>>, one borrow_mut per watcher step.
mod harness;
use std::cell::RefCell;
use std::rc::Rc;

const P: usize = 1 << 18;
const W: usize = 1 << 20;
const FRAMES: usize = 32;

struct Player { hp: i32, x: i32, inv: Vec<i32> }

fn main() {
    let t0 = harness::start();
    let mut rng = harness::Rng::new(42);
    let players: Vec<Rc<RefCell<Player>>> = (0..P)
        .map(|i| Rc::new(RefCell::new(Player { hp: 1000, x: (i & 1023) as i32, inv: Vec::new() })))
        .collect();
    let mut ws: Vec<Rc<RefCell<Player>>> = Vec::with_capacity(W);
    for _ in 0..W { ws.push(Rc::clone(&players[rng.next() as usize & (P - 1)])); }
    let mut acc: u64 = 0;
    for f in 0..FRAMES {
        for i in 0..W {
            {
                let mut t = ws[i].borrow_mut();
                t.hp = t.hp.wrapping_sub(1);
                acc = acc.wrapping_add((t.x as u32 as u64).wrapping_mul(t.inv.len() as u64 + 1));
                if (i & 7) == (f & 7) {
                    t.inv.push(f as i32);
                    if t.inv.len() > 8 {
                        for v in t.inv.iter() { acc = acc.wrapping_add(*v as u32 as u64); }
                        t.inv.clear();
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
        let p = p.borrow();
        sum = sum.wrapping_add(p.hp as u32 as u64).wrapping_add(p.inv.len() as u64);
    }
    harness::report("share-rust-refcell", t0, sum);
}
