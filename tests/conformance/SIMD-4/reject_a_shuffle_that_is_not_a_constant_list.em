#$ test: compile-fail
#$ rules: SIMD-4
# `[SIMD-4]` — a shuffle's lanes are "a constant lane list": one constant lane number per lane,
# each below the lane count.
from std.simd import i32x4

fn main():
    v = i32x4.splat(1)
    n = 2
    a = v.shuffle([0, 1, n, 3])  #$ error[E2020]: `shuffle` takes a list of 4 constant lane numbers
    b = v.shuffle([0, 1, 2])  #$ error[E2020]: `shuffle` takes a list of 4 constant lane numbers
    c = v.shuffle([0, 1, 2, 4])  #$ error[E2020]: each below 4
