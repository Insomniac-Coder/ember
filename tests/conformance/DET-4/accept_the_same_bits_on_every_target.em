#$ test: run-pass
#$ rules: DET-4, DET-2, TYP-9
#$ profiles: debug, release, shipping
#$ stdout: 0.8414709848078965 0.5403023058681398 1.5574077246549023 -0.479425538604203 -0.9899924966004454 14.101419947171719
#$ 2.7182818284590455 2.7536449349747158e-05 2.302585092994046 -6.907755278982137 1.4142135623730951 0.00019952623149688788
#$ 0.7853981633974483 2.356194490192345 -2.5535900500422257 1.4142135623730951
#$ -0.8522008497671888 -0.5753861119575491 0.7936137110620269 1.0
#$ -inf nan inf 0.0 -512.0 nan
#$ inf -inf 1.0 3.141592653589793 -0.0 -0.0
#$ 0.47942555 0.87758255 0.5463025 1.6487212 -0.6931472 0.70710677 2.3561945 0.70710677
# `[DET-4]` — `std.math.det`'s functions give the same bits on every target,
# so every digit printed here is the language's, not the platform's (a test of
# `std.math`'s may print none of them). They are fdlibm's algorithms in
# arithmetic Ember never fuses or reorders (`[TYP-9]`); an `f32` is computed
# in `f64` and rounded once. The special cases are C's: `log(0)` is `-inf`,
# a negative number to a non-integer power `nan`, `pow(1, nan)` 1.

import math.det

fn main():
    println(det.sin(1.0), det.cos(1.0), det.tan(1.0), det.sin(-0.5), det.cos(3.0), det.tan(1.5))
    println(det.exp(1.0), det.exp(-10.5), det.log(10.0), det.log(0.001), det.pow(2.0, 0.5), det.pow(10.0, -3.7))
    println(det.atan2(1.0, 1.0), det.atan2(1.0, -1.0), det.atan2(-2.0, -3.0), det.sqrt(2.0))
    # A huge argument is reduced exactly (Payne and Hanek): these are the
    # sines of the numbers written. 5.319372648326541e+255 is the double
    # nearest a multiple of π/2 (6381956970095103 × 2^797).
    println(det.sin(1e22), det.cos(1e300), det.tan(1647100.0), det.sin(5.319372648326541e+255))
    # C's special cases.
    println(det.log(0.0), det.log(-1.0), det.exp(1000.0), det.exp(-1000.0), det.pow(-8.0, 3.0), det.pow(-8.0, 1.0 / 3.0))
    println(det.pow(0.0, -1.0), det.pow(-0.0, -3.0), det.pow(1.0, f64.NAN), det.atan2(0.0, -0.0), det.atan2(-0.0, 1.0), det.sin(-0.0))
    # `f32` in, `f32` out: computed in `f64` and rounded once.
    x: f32 = 0.5
    println(det.sin(x), det.cos(x), det.tan(x), det.exp(x), det.log(x), det.pow(x, x), det.atan2(x, -x), det.sqrt(x))
