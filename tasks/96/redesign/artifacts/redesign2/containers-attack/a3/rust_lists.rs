// Safe Rust twin of alias_lists.c: copy_alive(&Vec, &mut Vec), which can never be one list.
include!("../rharness.rs");
#[derive(Clone, Copy)]
#[allow(dead_code)]
struct Enemy { id: u32, hp: i32, x: f32, vx: f32 }
const N: u32 = 1_000_000;
#[inline(never)]
fn copy_alive(src: &Vec<Enemy>, dst: &mut Vec<Enemy>) {
    for e in src.iter() { if e.hp > 0 { dst.push(*e); } }
}
fn main() {
    let t0 = h::start();
    h::seed(5);
    let mut a: Vec<Enemy> = Vec::new();
    for i in 0..N { a.push(Enemy { id: i, hp: (h::rand() % 10) as i32 - 1, x: (i % 100) as f32, vx: 1.0 }); }
    let mut b: Vec<Enemy> = Vec::new();
    let mut s: u64 = 0;
    for _ in 0..100 {
        b.clear();
        copy_alive(std::hint::black_box(&a), std::hint::black_box(&mut b));
        s += b.len() as u64 + b[b.len() / 2].id as u64;
    }
    h::report("a3-lists-rust", t0, s);
}
