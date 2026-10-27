// rust_hotshare_arc.rs: rust_hotshare_rc.rs with Arc for Rc (one fetch_add and a test of its result against
// isize::MAX per clone, a fetch_sub(Release) per drop). Every step clones an Arc of a cache-resident player
// into a watcher's slot and drops the Arc it held. The same Player as the retarget
// programs (Cell / RefCell fields), the same order of work, checksum 41,904,175,104.
mod harness;
use std::cell::{Cell, RefCell};
use std::sync::Arc;

const HP: usize = 1024;
const HW: usize = 4096;
const HFRAMES: usize = 20_000;

#[allow(dead_code)]
struct Player { hp: Cell<i32>, x: i32, inv: RefCell<Vec<i32>> }

fn main() {
    let t0 = harness::start();
    let players: Vec<Arc<Player>> = (0..HP)
        .map(|i| Arc::new(Player { hp: Cell::new(1000), x: i as i32, inv: RefCell::new(Vec::new()) }))
        .collect();
    let mut ws: Vec<Arc<Player>> = (0..HW).map(|i| Arc::clone(&players[i & (HP - 1)])).collect();
    let mut acc: u64 = 0;
    for f in 0..HFRAMES {
        for i in 0..HW {
            let t = Arc::clone(&players[(3 * i + f) & (HP - 1)]);
            acc = acc.wrapping_add(t.x as u32 as u64);
            ws[i] = t;                                   // drops the old Arc
        }
    }
    let mut sum = acc;
    for w in ws.iter() { sum = sum.wrapping_add(w.x as u32 as u64); }
    harness::report("rust-hotshare-arc", t0, sum);
}
