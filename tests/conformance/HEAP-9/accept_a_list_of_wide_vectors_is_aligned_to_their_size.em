#$ test: run-pass
#$ rules: HEAP-9
#$ profiles: debug, development, release
#$ assert-c: contains("ember_vec_push_aligned")
#$ stdout: 32 64 0 0 0 0
# `[HEAP-9]` — a list's storage is aligned to its element type's alignment however large: a SIMD
# vector wider than 16 bytes is aligned to its size (`[SIMD-1]`), and so is every buffer of a list
# of them, from the first push through growth and a clone.
from std.simd import f32x8, f64x8

fn off8(xs: Span[f32x8]) -> int:
    p: *f32x8 = xs.as_ptr()
    unsafe:  # SAFETY: only the address is read, as a number; nothing is accessed through it
        return ((p as usize) % 32) as int

fn off64(xs: Span[f64x8]) -> int:
    p: *f64x8 = xs.as_ptr()
    unsafe:  # SAFETY: only the address is read, as a number; nothing is accessed through it
        return ((p as usize) % 64) as int

fn main():
    xs: Array[f32x8] = Array[f32x8]()
    xs.push(f32x8.splat(1.0))
    first = off8(xs.as_span())
    for i in 0..300:
        xs.push(f32x8.splat(i as f32))
    copy = xs.clone()
    ys = [f64x8.splat(1.0), f64x8.splat(2.0)]
    println(align_of[f32x8](), align_of[f64x8](), first, off8(xs.as_span()), off8(copy.as_span()), off64(ys.as_span()))
