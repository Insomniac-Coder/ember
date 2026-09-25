#$ test: run-pass
#$ rules: STD-20, STD-3
#$ profiles: debug, release, shipping
#$ stdout: 1.4142135623730951 1.4142135 1024.0 2.5
#$ 2.0 4.0 3.0 -2.0 2.0 -1.0 -0.75
#$ 1.0 0.0 3.0 3.0 3.141592653589793
#$ true true true 3.0 -2.0 7.0
# `[STD-20]` — the float methods: `round` is half to even, as Python's
# (`2.5` to `2.0`), `round_half_away` is C's `round`, `fract` keeps the sign,
# `mul_add` is one rounding (`[STD-3]`). Each is the C library's function of
# its width (`sqrtf` for an `f32`).

fn main():
    x = 2.0
    y: f32 = 2.0
    println(x.sqrt(), y.sqrt(), x.pow(10.0), y.hypot(1.5))
    println((2.5).round(), (3.5).round(), (2.5).round_half_away(), (-1.25).floor(), (1.25).ceil(), (-1.75).trunc(), (-1.75).fract())
    println((0.0).exp(), (1.0).ln(), (8.0).log2(), (1000.0).log10(), (1.0).atan2(1.0) * 4.0)
    println((0.0 / 0.0).is_nan(), (1.0 / 0.0).is_infinite(), (1.0).is_finite(), (-3.0).abs(), (2.0).copysign(-1.0), (2.0).mul_add(3.0, 1.0))
