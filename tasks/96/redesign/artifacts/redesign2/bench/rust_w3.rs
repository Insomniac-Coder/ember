// Safe Rust, W3 (game objects that refer to each other), written the way Rust game code does it:
// objects in an arena of generational slots (the `slotmap` idea, written out so nothing is
// downloaded), targets as {index, generation} keys checked on use, so a destroyed target reads None
// and nothing dangles. Updates are chosen at run time through a table of functions indexed by kind
// (as plain C's table); each takes the world and the object's index, because a Turret writes another
// object's health while it is being updated, which `&mut self` plus `&mut target` would refuse.
#[path = "rharness.rs"]
mod harness;

const M: i64 = 100_000;

#[derive(Clone, Copy, PartialEq)]
struct Key { idx: u32, gen: u32 }
#[derive(Clone, Copy)]
struct Obj { id: i64, kind: u32, x: i32, y: i32, health: i32, target: Option<Key> }
struct Slot { gen: u32, obj: Obj } // gen odd: live
struct World { slots: Vec<Slot>, free: Vec<u32> }

impl World {
    fn get(&self, k: Key) -> Option<&Obj> {
        match self.slots.get(k.idx as usize) { Some(s) if s.gen == k.gen => Some(&s.obj), _ => None }
    }
    fn create(&mut self, obj: Obj) -> Key {
        if let Some(i) = self.free.pop() {
            let s = &mut self.slots[i as usize];
            s.gen += 1;
            s.obj = obj;
            Key { idx: i, gen: s.gen }
        } else {
            self.slots.push(Slot { gen: 1, obj });
            Key { idx: self.slots.len() as u32 - 1, gen: 1 }
        }
    }
    fn destroy(&mut self, k: Key) {
        let s = &mut self.slots[k.idx as usize];
        if s.gen != k.gen { panic!("destroy: stale key"); }
        s.gen += 1;
        if s.gen < u32::MAX - 1 { self.free.push(k.idx); } // retire an exhausted slot
    }
}

fn sign(d: i32) -> i32 { (d > 0) as i32 - (d < 0) as i32 }
fn ship(w: &mut World, me: usize) {
    let o = w.slots[me].obj;
    if let Some(t) = o.target.and_then(|t| w.get(t)).map(|t| (t.x, t.y)) {
        let s = &mut w.slots[me].obj;
        s.x += sign(t.0 - o.x);
        s.y += sign(t.1 - o.y);
    }
}
fn rock(w: &mut World, me: usize) { let s = &mut w.slots[me].obj; s.x = (s.x + 1) % 1000; }
fn turret(w: &mut World, me: usize) {
    let o = w.slots[me].obj;
    if let Some(t) = o.target {
        if let Some(to) = w.get(t) {
            if (to.x - o.x).abs() + (to.y - o.y).abs() < 300 { w.slots[t.idx as usize].obj.health -= 1; }
        }
    }
}
const UPDATE: [fn(&mut World, usize); 3] = [ship, rock, turret];

fn make(rng: &mut harness::Rng, id: i64) -> Obj {
    let x = (rng.next() % 1000) as i32;
    let y = (rng.next() % 1000) as i32;
    Obj { id, kind: (id % 3) as u32, x, y, health: (100 + id % 50) as i32, target: None }
}

fn main() {
    let t0 = harness::start();
    let mut rng = harness::Rng::new(7);
    let mut w = World { slots: Vec::new(), free: Vec::new() };
    let mut live: Vec<Key> = Vec::new(); // in increasing id order
    for i in 0..M {
        let mut o = make(&mut rng, i);
        if i != 0 { o.target = Some(live[(rng.next() % i as u32) as usize]); }
        let k = w.create(o);
        live.push(k);
    }
    let mut next_id = M;
    let mut destroyed: u64 = 0;
    for _ in 0..1000 {
        for i in 0..live.len() {
            let me = live[i].idx as usize;
            let kind = w.slots[me].obj.kind as usize;
            UPDATE[kind](&mut w, me);
        }
        let before = live.len();
        let mut j = 0;
        for i in 0..before {
            let k = live[i];
            if w.slots[k.idx as usize].obj.health <= 0 { w.destroy(k); } else { live[j] = k; j += 1; }
        }
        live.truncate(j);
        let dead = before - j;
        destroyed += dead as u64;
        for _ in 0..dead {
            let mut o = make(&mut rng, next_id);
            next_id += 1;
            if !live.is_empty() { o.target = Some(live[(rng.next() % live.len() as u32) as usize]); }
            let k = w.create(o);
            live.push(k);
        }
    }
    let mut s: i64 = 0;
    for k in &live { let o = &w.slots[k.idx as usize].obj; s += o.x as i64 + o.y as i64 + o.health as i64; }
    let _ = live.first().map(|k| w.slots[k.idx as usize].obj.id);
    harness::report("rust-w3", t0, (s as u64).wrapping_add(1_000_000 * destroyed));
}
