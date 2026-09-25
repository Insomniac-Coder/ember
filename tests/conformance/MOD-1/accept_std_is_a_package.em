#$ test: run-pass
#$ rules: MOD-1, STD-1
## "Module path = package name + path from `src/`". `std` is a package, so
## `std.math` is `math.em` under **its** `src/`, not `std/math.em` under this
## one — which is what makes a `std` shipped beside the compiler resolvable.

from std.math import lerp

fn main():
    println(lerp(0.0, 100.0, 0.5))
#$ stdout: 50.0
