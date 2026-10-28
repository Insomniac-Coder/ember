// rust_respan.rs: workload respan in safe Rust. Argument `repoint`: the callee takes the caller's
// slice by `&mut &mut [f32]`, which lets it point the caller's slice elsewhere (Rust's form of a
// `ref mut` span), so after the call the caller reads the slice's words again. Argument `plain`: the
// idiom, `&mut [f32]` (two words; the callee cannot re-point it), as the model's -DWORDS.
// The callee is called through a function pointer hidden by `black_box`, so the caller cannot see its
// body (the C programs put it in another object file); the call is indirect here, direct in C.
// Build: rustc -C opt-level=2 --edition 2021 rust_respan.rs
mod harness;

const POOL: usize = 4096;
const CHUNK: usize = 16;
const FRAMES: usize = 117188;

#[inline(never)]
fn integrate_repoint(pos: &mut &mut [f32], vel: &[f32], dt: f32) {
    for (p, &v) in pos.iter_mut().zip(vel) { *p += v * dt; }
}

#[inline(never)]
fn integrate(pos: &mut [f32], vel: &[f32], dt: f32) {
    for (p, &v) in pos.iter_mut().zip(vel) { *p += v * dt; }
}

#[inline(always)]
fn wrap(s: &mut [f32]) {
    for x in s.iter_mut() {
        if *x > 1000.0 { *x -= 2000.0; } else if *x < -1000.0 { *x += 2000.0; }
    }
}

fn main() {
    let t0 = harness::start();
    let repoint = std::env::args().nth(1).map_or(false, |a| a == "repoint");
    let mut rng = harness::Rng::new(37);
    let mut pos = vec![0f32; POOL];
    let mut vel = vec![0f32; POOL];
    for i in 0..POOL {
        pos[i] = ((rng.next() % 2001) as i32 - 1000) as f32;
        vel[i] = ((rng.next() % 2001) as i32 - 1000) as f32 * 0.01;
    }
    let dt = 0.01f32;
    let f_repoint: fn(&mut &mut [f32], &[f32], f32) = std::hint::black_box(integrate_repoint);
    let f_plain: fn(&mut [f32], &[f32], f32) = std::hint::black_box(integrate);
    for _ in 0..FRAMES {
        for (pc, vc) in pos.chunks_exact_mut(CHUNK).zip(vel.chunks_exact(CHUNK)) {
            if repoint {
                let mut s: &mut [f32] = pc;
                f_repoint(&mut s, vc, dt);
                wrap(s);
            } else {
                f_plain(pc, vc, dt);
                wrap(pc);
            }
        }
    }
    let mut sum: u64 = 0;
    for &x in &pos { sum = sum.wrapping_add((x * 1000.0) as i64 as u64); }
    harness::report(if repoint { "rust_respan-repoint" } else { "rust_respan-plain" }, t0, sum);
}
