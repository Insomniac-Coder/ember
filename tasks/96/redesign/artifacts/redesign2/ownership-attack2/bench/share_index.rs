// share_index.rs: the shared-object workload in safe Rust as a Rust programmer writing for speed
// writes it: players in one Vec, each watcher a position (no Rc, no RefCell, bounds-checked indexing).
// Same work, same random numbers, same checksum as share.c.
mod harness;
const P: usize = 1 << 18;
const W: usize = 1 << 20;
const FRAMES: usize = 32;
struct Player { hp: i32, x: i32, inv: Vec<i32> }
fn main() {
    let t0 = harness::start();
    let mut rng = harness::Rng::new(42);
    let mut players: Vec<Player> = (0..P).map(|i| Player { hp: 1000, x: (i & 1023) as i32, inv: Vec::new() }).collect();
    let mut ws: Vec<u32> = Vec::with_capacity(W);
    for _ in 0..W { ws.push(rng.next() & (P as u32 - 1)); }
    let mut acc: u64 = 0;
    for f in 0..FRAMES {
        for i in 0..W {
            {
                let t = &mut players[ws[i] as usize];
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
            if rng.next() & 15 == 0 { ws[i] = rng.next() & (P as u32 - 1); }
        }
    }
    let mut sum = acc;
    for p in players.iter() { sum = sum.wrapping_add(p.hp as u32 as u64).wrapping_add(p.inv.len() as u64); }
    harness::report("share-rust-index", t0, sum);
}
