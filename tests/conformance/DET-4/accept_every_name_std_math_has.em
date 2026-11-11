#$ test: run-pass
#$ rules: DET-4, STD-21
#$ profiles: debug, development, release
#$ stdout: 0.5235987755982989 1.0471975511965979 0.7853981633974483
#$ 1.1752011936438016 1.5430806348152437 0.46211715726000974
#$ 1448.1546878700492 3.321928094887362 3.0
#$ 3.0 -1.2599210498948732 5.0 1.414213562373095e+200
#$ 0.5235988 1.0471976 0.4636476 0.5210953 1.127626 0.46211717
#$ 1.4142135 -1.0 -0.30103 0.7937005 0.70710677
# `[DET-4]` — `std.math.det` provides, under the names `[STD-21]` gives them, every function of
# `std.math` that is `Nondet`: `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `sinh`, `cosh`,
# `tanh`, `exp`, `exp2`, `ln`, `log2`, `log10`, `pow`, `cbrt`, `hypot`, and `sqrt` beside them. Each
# takes `f32` or `f64` and answers in it.

import math.det

fn main():
    println(det.asin(0.5), det.acos(0.5), det.atan(1.0))
    println(det.sinh(1.0), det.cosh(1.0), det.tanh(0.5))
    println(det.exp2(10.5), det.log2(10.0), det.log10(1000.0))
    println(det.cbrt(27.0), det.cbrt(-2.0), det.hypot(3.0, 4.0), det.hypot(1e200, 1e200))
    x: f32 = 0.5
    println(det.asin(x), det.acos(x), det.atan(x), det.sinh(x), det.cosh(x), det.tanh(x))
    println(det.exp2(x), det.log2(x), det.log10(x), det.cbrt(x), det.hypot(x, x))
