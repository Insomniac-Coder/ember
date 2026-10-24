// W1 (ECS) in safe Rust, EnTT layout, the same structure as plain_sparse.c: one sparse set per
// component, walk the Velocity pool backwards by index, probe Position's sparse array, swap-remove a
// despawned entity from both pools, append spawns. It compiles in safe Rust because the loop holds
// indices, not references, across the changes. Every index is bounds-checked unless LLVM proves it.
include!("../rharness.rs");
const N: u32 = 1_000_000;
const NONE: u32 = u32::MAX;
#[derive(Clone, Copy)]
struct V2 { x: f32, y: f32 }
struct Pool { sparse: Vec<u32>, ents: Vec<u32>, data: Vec<V2> }
impl Pool {
    fn new() -> Pool { Pool { sparse: Vec::new(), ents: Vec::new(), data: Vec::new() } }
    fn grow_sparse(&mut self, id: u32) {
        let id = id as usize;
        if id < self.sparse.len() { return; }
        let mut nc = if self.sparse.len() > 0 { self.sparse.len() } else { 1024 };
        while nc <= id { nc *= 2; }
        self.sparse.resize(nc, NONE);
    }
    fn add(&mut self, id: u32, v: V2) {
        self.grow_sparse(id);
        self.sparse[id as usize] = self.ents.len() as u32;
        self.ents.push(id);
        self.data.push(v);
    }
    fn del(&mut self, id: u32) {
        let i = self.sparse[id as usize] as usize;
        let last = *self.ents.last().unwrap();
        self.ents.swap_remove(i);
        self.data.swap_remove(i);
        if i < self.ents.len() { self.sparse[last as usize] = i as u32; }
        self.sparse[id as usize] = NONE;
    }
}
fn main() {
    let t0 = h::start();
    let mut pos = Pool::new();
    let mut vel = Pool::new();
    for i in 0..N {
        pos.add(i, V2 { x: (i % 1000) as f32, y: (i / 1000) as f32 });
        if i % 2 == 0 { vel.add(i, V2 { x: (i % 7) as f32 * 0.5, y: (i % 5) as f32 * 0.25 }); }
    }
    let mut next = N;
    for s in 0..100u32 {
        let mut j = vel.ents.len();
        while j > 0 {
            j -= 1;
            let id = vel.ents[j];
            let pi = if (id as usize) < pos.sparse.len() { pos.sparse[id as usize] } else { NONE };
            if pi == NONE { continue; }
            let v = vel.data[j];
            let p = &mut pos.data[pi as usize];
            p.x += v.x; p.y += v.y;
            if id % 100 == s {
                pos.del(id); vel.del(id);
                let e = next; next += 1;
                pos.add(e, V2 { x: 0.0, y: 0.0 }); vel.add(e, V2 { x: 1.0, y: 1.0 });
            }
        }
    }
    let mut sum: u64 = 0;
    for d in &pos.data { sum = sum.wrapping_add((((d.x * 4.0) as i64) + ((d.y * 4.0) as i64)) as u64); }
    h::report("ecs-rust-sparse", t0, sum.wrapping_add(pos.data.len() as u64));
}
