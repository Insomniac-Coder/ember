// Safe Rust, W1 (ECS) in EnTT's owning-group layout: the same data and the same algorithm as
// plain/w1_group.c. Despawn and spawn are written inside the loop body. A Rust programmer cannot
// hold `&mut` into the pools across `world.destroy(e)`, so the loop walks an index and indexes the
// pools (bounds-checked) on every step: the safe spelling of this program.
#[path = "rharness.rs"]
mod harness;

const N: u32 = 1_000_000;
const NONE: u32 = u32::MAX;

#[derive(Clone, Copy)]
struct V2 { x: f32, y: f32 }

#[derive(Default)]
struct Pool { sparse: Vec<u32>, ents: Vec<u32>, data: Vec<V2> }

impl Pool {
    fn has(&self, id: u32) -> bool { (id as usize) < self.sparse.len() && self.sparse[id as usize] != NONE }
    fn add(&mut self, id: u32, v: V2) {
        if id as usize >= self.sparse.len() {
            let mut nc = if self.sparse.is_empty() { 1024 } else { self.sparse.len() };
            while nc <= id as usize { nc *= 2; }
            self.sparse.resize(nc, NONE);
        }
        self.sparse[id as usize] = self.ents.len() as u32;
        self.ents.push(id);
        self.data.push(v);
    }
    fn swap_slots(&mut self, i: usize, j: usize) {
        if i == j { return; }
        self.ents.swap(i, j);
        self.data.swap(i, j);
        let (a, b) = (self.ents[i], self.ents[j]);
        self.sparse[a as usize] = i as u32;
        self.sparse[b as usize] = j as u32;
    }
    fn del(&mut self, id: u32) {
        let i = self.sparse[id as usize] as usize;
        self.ents.swap_remove(i);
        self.data.swap_remove(i);
        if i < self.ents.len() { let m = self.ents[i]; self.sparse[m as usize] = i as u32; }
        self.sparse[id as usize] = NONE;
    }
}

struct World { pos: Pool, vel: Pool, glen: usize, next: u32 }

impl World {
    fn create(&mut self) -> u32 { self.next += 1; self.next - 1 }
    fn group_enter(&mut self, id: u32) {
        let g = self.glen;
        let (pi, vi) = (self.pos.sparse[id as usize] as usize, self.vel.sparse[id as usize] as usize);
        self.pos.swap_slots(pi, g);
        self.vel.swap_slots(vi, g);
        self.glen += 1;
    }
    fn group_leave(&mut self, id: u32) {
        self.glen -= 1;
        let g = self.glen;
        let (pi, vi) = (self.pos.sparse[id as usize] as usize, self.vel.sparse[id as usize] as usize);
        self.pos.swap_slots(pi, g);
        self.vel.swap_slots(vi, g);
    }
    fn emplace_pos(&mut self, id: u32, v: V2) { self.pos.add(id, v); if self.vel.has(id) { self.group_enter(id); } }
    fn emplace_vel(&mut self, id: u32, v: V2) { self.vel.add(id, v); if self.pos.has(id) { self.group_enter(id); } }
    fn destroy(&mut self, id: u32) {
        let (hp, hv) = (self.pos.has(id), self.vel.has(id));
        if hp && hv && (self.pos.sparse[id as usize] as usize) < self.glen { self.group_leave(id); }
        if hp { self.pos.del(id); }
        if hv { self.vel.del(id); }
    }
}

fn main() {
    let t0 = harness::start();
    let mut w = World { pos: Pool::default(), vel: Pool::default(), glen: 0, next: 0 };
    for i in 0..N {
        let e = w.create();
        w.emplace_pos(e, V2 { x: (i % 1000) as f32, y: (i / 1000) as f32 });
        if i % 2 == 0 { w.emplace_vel(e, V2 { x: (i % 7) as f32 * 0.5, y: (i % 5) as f32 * 0.25 }); }
    }
    for s in 0..100u32 {
        let mut k = w.glen;
        while k > 0 {
            k -= 1;
            let e = w.pos.ents[k];
            let v = w.vel.data[k];
            let p = &mut w.pos.data[k];
            p.x += v.x;
            p.y += v.y;
            if e % 100 == s {
                w.destroy(e);
                let n = w.create();
                w.emplace_pos(n, V2 { x: 0.0, y: 0.0 });
                w.emplace_vel(n, V2 { x: 1.0, y: 1.0 });
            }
        }
    }
    let mut sum: u64 = 0;
    for p in &w.pos.data { sum = sum.wrapping_add(((p.x * 4.0) as i64 + (p.y * 4.0) as i64) as u64); }
    harness::report("rust-w1", t0, sum + w.pos.data.len() as u64);
}
