// rust_weakview_rc.rs: rust_weakview_arc.rs with Rc and rc::Weak, the one-thread reference (what
// model_weakview_onethread.c is to model_weakview.c). Rc's Weak::upgrade is a test of the strong count
// and a plain +1 (checked for wrap-around), the view's drop a plain -1 and a test for zero. Same work,
// same order; checksum 33,790,885,888.
mod harness;
use std::rc::{Rc, Weak};

const WP: usize = 1 << 14;
const WC: usize = 1 << 20;
const WF: usize = 64;
const WMULT: usize = 40503;

#[allow(dead_code)]
struct Player { x: i64, id: i64 }

fn main() {
    let t0 = harness::start();
    let mut players: Vec<Rc<Player>> = (0..WP).map(|j| Rc::new(Player { x: (j & 1023) as i64, id: j as i64 })).collect();
    let mut cams: Vec<Weak<Player>> = (0..WC).map(|i| Rc::downgrade(&players[(i * WMULT) & (WP - 1)])).collect();
    let mut acc: u64 = 0;
    let mut fails: u64 = 0;
    for f in 0..WF {
        let r = f & 63;
        let mut j = r;
        while j < WP {
            players[j] = Rc::new(Player { x: (j & 1023) as i64, id: (WP * (f + 1) + j) as i64 });
            j += 64;
        }
        for i in 0..WC {
            let s = (i * WMULT) & (WP - 1);
            match cams[i].upgrade() {
                Some(p) => acc = acc.wrapping_add(p.x as u64),
                None => { cams[i] = Rc::downgrade(&players[s]); fails += 1; }
            }
        }
    }
    harness::report("rust-weakview-rc", t0, acc.wrapping_add(fails));
}
