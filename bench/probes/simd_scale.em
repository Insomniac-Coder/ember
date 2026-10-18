# Part XII.2's `scale` (`[SIMD-1]`, `[SIMD-9]`): 4096 floats scaled eight at a time through
# `f32x8`, 200 thousand times, the C twin's plain loop against it.
from std.simd import f32x8

fn scale(xs: Span[f32], mut out: MutSpan[f32], k: f32):
    assert(xs.len() == out.len() and xs.len() % 8 == 0)
    kv = f32x8.splat(k)
    for i in (0..xs.len()).step_by(8):
        v = f32x8.load(xs[i..i + 8])
        (v * kv).store(out[i..i + 8])

fn main():
    n = 4096
    xs: Array[f32] = []
    out: Array[f32] = []
    for i in 0..n:
        xs.push((i % 100) as f32)
        out.push(0.0)
    k: f32 = 1.0
    for r in 0..200_000:
        scale(xs, out, k)
        k = out[(r * 7) % n] * 0.0001 + 1.0
    total = 0.0
    for x in out:
        total += x as f64
    println(f"{total:.1f}")
