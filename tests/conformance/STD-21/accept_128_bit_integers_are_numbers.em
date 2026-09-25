#$ test: run-pass
#$ rules: STD-21, IFC-4, TYP-1
#$ stdout: 1.3043817825332783e+19 1.8446744073709552e+19 3.0 1.7014118346046923e+38
# D-272 — `i128` and `u128` are numbers to `std.math` like every other
# integer (ODR-038): each answers in `f64`, rounded to nearest. `sqrt` is
# correctly rounded on every platform (IEEE 754), so its digits are exact.

import math

fn main():
    hi: i128 = 170141183460469231731687303715884105727
    top: u128 = 340282366920938463463374607431768211455
    nine: i128 = 9
    println(math.sqrt(hi), math.sqrt(top), math.sqrt(nine), hi.to_real())
