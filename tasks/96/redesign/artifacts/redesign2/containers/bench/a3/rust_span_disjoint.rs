// Safe Rust for span_add.c, second spelling: get_disjoint_mut([a..a+len, b..b+len]) checks that the
// two ranges are in bounds and do not overlap, and gives two &mut slices (noalias) to an out-of-line
// dst[k] = dst[k] * 0.5 + src[k]. First spelling: rust_span.rs (split_at_mut with an order test).
// The length is read through black_box, so it is unknown to the compiler.
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
        match pool.get_disjoint_mut([a..a + len, b..b + len]) {
            Ok([d, s]) => add(d, s),
            Err(_) => panic!("overlap"),
        }
    }
    let mut s: u64 = 0;
    for k in 0..4096 { s = s.wrapping_add(((pool[k] * 1000.0) as i64) as u64); }
    h::report("a3-span-rust-disjoint", t0, s);
}
