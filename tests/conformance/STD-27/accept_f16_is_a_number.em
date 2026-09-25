#$ test: run-pass
#$ rules: STD-27, STD-21
#$ profiles: debug, release, shipping
#$ stdout: 1.4142135 6.0 2.5
# ODR-041 — `f16` is a `Number` whose answers come back as `f32`, which
# holds every `f16` exactly: `math.sqrt` of an `f16` is an `f32`.

import math

fn main():
    x: f16 = 2.0
    root: f32 = math.sqrt(x)
    println(root, math.fma(x, x, x), x.to_real() + 0.5)
