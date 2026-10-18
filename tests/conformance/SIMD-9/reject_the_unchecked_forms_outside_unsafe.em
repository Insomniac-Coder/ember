#$ test: compile-fail
#$ rules: SIMD-9
# `[SIMD-9]` — "The unchecked forms (`load_unchecked`, `store_unchecked`, `gather_unchecked`,
# `scatter_unchecked`) are `unsafe fn`": each needs an `unsafe` block.
from std.simd import f32x4, i32x4

fn main():
    xs: Array[f32] = [1.0, 2.0, 3.0, 4.0]
    idx = i32x4.splat(0)
    v = f32x4.load_unchecked(xs)  #$ error[E3100]: `load_unchecked` needs an `unsafe` block
    v.store_unchecked(xs)  #$ error[E3100]: `store_unchecked` needs an `unsafe` block
    w = f32x4.gather_unchecked(xs, idx)  #$ error[E3100]: `gather_unchecked` needs an `unsafe` block
    w.scatter_unchecked(xs, idx)  #$ error[E3100]: `scatter_unchecked` needs an `unsafe` block
