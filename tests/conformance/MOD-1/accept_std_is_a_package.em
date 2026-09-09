#$ test: run-pass
#$ rules: MOD-1, STD-1
## "Module path = package name + path from `src/`". `std` is a package, so
## `std.math` is `math.em` under **its** `src/`, not `std/math.em` under this
## one — which is what makes a `std` shipped beside the compiler resolvable.

from std.math import clamp_i32

fn main():
    println(clamp_i32(200, 0, 100))
#$ stdout: 100
