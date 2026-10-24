// Safe Rust doing the same work the same way as plain.c: the in-place filter with two indices over
// the live length, each element copied out once, updated, and stored once at w (Enemy is Copy). It
// compiles in safe Rust because no reference is held across the push. Every index is bounds-checked
// unless LLVM proves it in range.
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
    for _f in 0..100 {
        let (mut r, mut w) = (0usize, 0usize);
        while r < v.len() {
            let mut e = v[r];
            e.x += e.vx; e.hp -= 1;
            if e.hp <= 0 {
                v.push(Enemy { id: next, hp: 2 + (h::rand() % 100) as i32, x: 0.0, vx: 1.0 });
                next += 1;
                r += 1;
                continue;
            }
            v[w] = e;
            w += 1; r += 1;
        }
        v.truncate(w);
    }
    let mut s: u64 = 0;
    for e in &v { s = s.wrapping_add((((e.x * 4.0) as i64) + e.hp as i64) as u64); }
    h::report("a4-rust-index", t0, s.wrapping_add(v.len() as u64 + next as u64));
}
