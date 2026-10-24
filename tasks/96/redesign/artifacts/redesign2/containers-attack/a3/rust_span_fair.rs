// Safe Rust for the span kernel (../../containers/bench/a3/span_add.c, length unknown), written so the
// order of the two spans does not become a branch: split once at the higher start, take both slices,
// then pick which is dst and which is src with a select. The report's rust_span.rs branches on a < b
// (random, so about half mispredict); its CALLCHECK model's first version had the same problem.
include!("../rharness.rs");
#[inline(never)]
fn add(dst: &mut [f32], src: &[f32]) {
    for (d, s) in dst.iter_mut().zip(src.iter()) { *d = *d * 0.5 + *s; }
}
fn main() {
    let t0 = h::start();
    let len: usize = std::hint::black_box(16);
    let mut pool = vec![0.0f32; 4096 + 16];
    h::seed(3);
    for k in 0..4096 + 16 { pool[k] = (h::rand() % 100) as f32 * 0.01; }
    for _ in 0..50_000_000u32 {
        let a = ((h::rand() % 128) * 32) as usize;
        let b = (a + 16 + ((h::rand() % 64) * 32) as usize) % 4096;
        let (lo, hi) = (a.min(b), a.max(b));
        let (x, y) = pool.split_at_mut(hi);
        let first = &mut x[lo..lo + len];
        let second = &mut y[..len];
        let (d, s) = if a < b { (first, second) } else { (second, first) };
        add(d, s);
    }
    let mut s: u64 = 0;
    for k in 0..4096 { s = s.wrapping_add(((pool[k] * 1000.0) as i64) as u64); }
    h::report("a3-span-rust-fair", t0, s);
}
