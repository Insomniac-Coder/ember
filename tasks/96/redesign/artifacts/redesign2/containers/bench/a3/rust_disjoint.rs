// Safe Rust's spelling of A3: get_disjoint_mut([i, j]) checks that i and j are distinct and in
// bounds, then gives two &mut (noalias).
include!("../rharness.rs");
#[derive(Clone, Copy)]
struct Body { x: f32, y: f32, vx: f32, vy: f32 }
const NB: usize = 4096;
#[inline]
fn collide(a: &mut Body, b: &mut Body) {
    let (tvx, tvy) = (a.vx, a.vy);
    a.vx = b.vx; a.vy = b.vy;
    b.vx = tvx; b.vy = tvy;
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    a.x -= dx * 0.01; a.y -= dy * 0.01;
    b.x += dx * 0.01; b.y += dy * 0.01;
}
fn main() {
    let t0 = h::start();
    h::seed(7);
    let mut bodies = vec![Body { x: 0.0, y: 0.0, vx: 0.0, vy: 0.0 }; NB];
    for k in 0..NB {
        let vx = (h::rand() % 7) as f32 - 3.0;
        let vy = (h::rand() % 5) as f32 - 2.0;
        bodies[k] = Body { x: (k % 64) as f32, y: (k / 64) as f32, vx, vy };
    }
    for _ in 0..50_000_000u32 {
        let i = (h::rand() as usize) % NB;
        let j = (i + 1 + (h::rand() as usize) % (NB - 1)) % NB;
        match bodies.get_disjoint_mut([i, j]) {
            Ok([a, b]) => collide(a, b),
            Err(_) => panic!("the same body passed twice"),
        }
    }
    let mut s: u64 = 0;
    for b in &bodies { s = s.wrapping_add((((b.x * 16.0) as i64) + ((b.vx * 16.0) as i64)) as u64); }
    h::report("a3-rust-disjoint", t0, s);
}
