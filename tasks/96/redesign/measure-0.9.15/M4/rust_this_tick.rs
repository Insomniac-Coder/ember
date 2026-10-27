// rust_this_tick.rs: workload this_tick in safe Rust. `tick(&mut self)`: `self` is noalias for
// LLVM (Rust's `&mut`), the comparison the area-B note asked for (run4/v5/area-B.md line 2521).
// Layout: `Vec<Box<Player>>`, each player its own heap block, as an Ember class object; Box blocks come
// from the system allocator (not carved from one block), and carry no 8-byte header.
// No overflow checks (Rust's release profile); `+=`/`-=` wrap silently there.
// Build: rustc -C opt-level=2 --edition 2021 rust_this_tick.rs
mod harness;

const PLAYERS: usize = 16384;
const FRAMES: usize = 12000;

pub struct Player { pub name: String, pub id: i64, pub hp: i64 }
pub struct Game {
    pub players: Vec<Box<Player>>,
    pub log: Vec<i64>,
    pub damage: i64, pub respawn_hp: i64, pub deaths: i64, pub total_hp: i64, pub frame: i64,
}

impl Game {
    #[allow(dead_code)]
    fn note(&mut self, who: i64) { self.log.push(who); }   // AN-17's `note`; see `tick`

    #[inline(never)]
    fn tick(&mut self) {
        self.log.clear();
        // `self.note(p.id)` inside the walk would borrow all of `self` while `p` borrows `players`
        // (E0499); the idiom inlines the push on the disjoint field, which is what `note` does.
        for p in self.players.iter_mut() {
            p.hp -= self.damage;
            if p.hp <= 0 {
                self.deaths += 1;
                p.hp = self.respawn_hp;
                self.log.push(p.id);
            }
            self.total_hp += p.hp;
        }
        self.frame += 1;
    }
}

fn main() {
    let t0 = harness::start();
    let mut rng = harness::Rng::new(17);
    let mut g = Game { players: Vec::with_capacity(PLAYERS), log: Vec::new(), damage: 1, respawn_hp: 1000, deaths: 0, total_hp: 0, frame: 0 };
    for i in 0..PLAYERS {
        g.players.push(Box::new(Player { name: String::new(), id: i as i64, hp: (rng.next() % 1000) as i64 + 1 }));
    }
    let mut logged: u64 = 0;
    for _ in 0..FRAMES {
        g.tick();
        for &k in &g.log { logged = logged.wrapping_add(k as u64); }
    }
    let sum = (g.total_hp as u64).wrapping_add((g.deaths as u64).wrapping_mul(1000003)).wrapping_add(logged).wrapping_add(g.frame as u64);
    harness::report("rust_this_tick", t0, sum);
}
