// Safe Rust, the best one-pass idiom: retain_mut updates and filters in one pass; spawns go to a
// second Vec (they cannot be pushed into `v` during retain_mut) and are appended after, with their
// in-frame update applied when made.
include!("../rharness.rs");
#[derive(Clone, Copy)]
#[allow(dead_code)]
struct Enemy { id: u32, hp: i32, x: f32, vx: f32 }
const N: u32 = 1_000_000;
fn main() {
    let t0 = h::start();
    h::seed(42);
    let mut v: Vec<Enemy> = Vec::new();
    for i in 0..N { v.push(Enemy { id: i, hp: 1 + (h::rand() % 100) as i32, x: (i % 1000) as f32, vx: (i % 7) as f32 * 0.5 }); }
    let mut next = N;
    let mut spawned: Vec<Enemy> = Vec::new();
    for _f in 0..100 {
        v.retain_mut(|e| {
            e.x += e.vx; e.hp -= 1;
            if e.hp <= 0 {
                spawned.push(Enemy { id: next, hp: 2 + (h::rand() % 100) as i32 - 1, x: 0.0 + 1.0, vx: 1.0 });
                next += 1;
                false
            } else { true }
        });
        v.extend(spawned.drain(..));
    }
    let mut s: u64 = 0;
    for e in &v { s = s.wrapping_add((((e.x * 4.0) as i64) + e.hp as i64) as u64); }
    h::report("a4-rust-retain-mut", t0, s.wrapping_add(v.len() as u64 + next as u64));
}
