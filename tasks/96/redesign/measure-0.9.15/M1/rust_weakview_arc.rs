// rust_weakview_arc.rs: weakview.h's workload in safe Rust. The players are Arc<Player> in a Vec (each
// player's one owner); a camera keeps a Weak<Player>. Opening is Weak::upgrade (a compare-exchange loop
// that raises the strong count only while it is above zero, with Arc's own overflow check), the view is
// the Arc it gives, and its drop is a fetch_sub(Release) (plus an Acquire fence on the last): exactly
// what [RC-5] point 2 asks Ember to do. Re-pointing a camera is Arc::downgrade (weak +1, checked) and the
// drop of the old Weak (weak -1). Same work, same order; checksum 33,790,885,888.
mod harness;
use std::sync::{Arc, Weak};

const WP: usize = 1 << 14;
const WC: usize = 1 << 20;
const WF: usize = 64;
const WMULT: usize = 40503;

#[allow(dead_code)]
struct Player { x: i64, id: i64 }

fn main() {
    let t0 = harness::start();
    let mut players: Vec<Arc<Player>> = (0..WP).map(|j| Arc::new(Player { x: (j & 1023) as i64, id: j as i64 })).collect();
    let mut cams: Vec<Weak<Player>> = (0..WC).map(|i| Arc::downgrade(&players[(i * WMULT) & (WP - 1)])).collect();
    let mut acc: u64 = 0;
    let mut fails: u64 = 0;
    for f in 0..WF {
        let r = f & 63;
        let mut j = r;
        while j < WP {
            players[j] = Arc::new(Player { x: (j & 1023) as i64, id: (WP * (f + 1) + j) as i64 });
            j += 64;
        }
        for i in 0..WC {
            let s = (i * WMULT) & (WP - 1);
            match cams[i].upgrade() {
                Some(p) => acc = acc.wrapping_add(p.x as u64),
                None => { cams[i] = Arc::downgrade(&players[s]); fails += 1; }
            }
        }
    }
    harness::report("rust-weakview-arc", t0, acc.wrapping_add(fails));
}
