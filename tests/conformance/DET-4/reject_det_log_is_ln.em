#$ test: compile-fail
#$ rules: DET-4
# `[DET-4]` — `det.log` is `E1010`, whose help is "the natural logarithm is `ln` in `std.math` and
# `std.math.det` alike: write `det.ln(p)`".

import math.det

fn main():
    p = 2.0
    println(det.log(p))    #$ error[E1010]: cannot find `std.math.det.log`
    #$ help: the natural logarithm is `ln` in `std.math` and `std.math.det` alike: write `det.ln(p)`
