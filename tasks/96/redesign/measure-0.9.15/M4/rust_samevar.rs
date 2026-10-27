// rust_samevar.rs: workload samevar in safe Rust. Rust refuses one `&mut` place given twice
// (E0499), so the idiom writes the one-name function by hand, `blend_one`: the same copy Ember's
// compiler makes (verdict 88), written by the programmer. `two` uses `blend` with two `&mut [f32]`
// (each noalias for LLVM). Slicing checks each range, as the model's span making does; the loops zip
// (no per-element checks; `zip` stops at the shorter slice, which the lengths here never reach).
// Argument `one` or `two`, as plain_samevar.c.
// Build: rustc -C opt-level=2 --edition 2021 rust_samevar.rs
mod harness;

const SPAN: usize = 16;
const POOL: usize = 4096;
const ROUNDS: usize = 195313;

#[inline(never)]
fn blend(dst: &mut [f32], src: &mut [f32]) {
    for (d, s) in dst.iter_mut().zip(src.iter_mut()) {
        *d = *d * 0.5 + *s * 0.25;
        *s = *s * 0.75 + 0.125;
    }
}

#[inline(never)]
fn blend_one(s: &mut [f32]) {
    for x in s.iter_mut() {
        *x = *x * 0.5 + *x * 0.25;
        *x = *x * 0.75 + 0.125;
    }
}

fn main() {
    let t0 = harness::start();
    let one = std::env::args().nth(1).map_or(false, |a| a == "one");
    let n: usize = std::hint::black_box(SPAN); // not known when compiling, as the C programs' volatile
    let mut rng = harness::Rng::new(31);
    let mut left = vec![0f32; POOL];
    let mut right = vec![0f32; POOL];
    for k in 0..POOL {
        left[k] = (rng.next() % 1000) as f32 * 0.001;
        right[k] = (rng.next() % 1000) as f32 * 0.001;
    }
    for r in 0..ROUNDS {
        for j in 0..256 {
            let a = j * SPAN;
            if one {
                blend_one(&mut left[a..a + n]);
            } else {
                let b = ((j * 97 + r) & 255) * SPAN;
                blend(&mut left[a..a + n], &mut right[b..b + n]);
            }
        }
    }
    let mut s: u64 = 0;
    for k in 0..POOL {
        s = s.wrapping_add((left[k] * 1000000.0) as i64 as u64).wrapping_add(((right[k] * 1000000.0) as i64 as u64).wrapping_mul(3));
    }
    harness::report(if one { "rust_samevar-one" } else { "rust_samevar-two" }, t0, s);
}
