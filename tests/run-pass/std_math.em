#$ test: run-pass
#$ rules: MOD-1, MOD-3, STD-1, STD-21

# `[MOD-1]` — `std` is a package, so `std.math` is `math.em` under its own
# `src/`, not `std/math.em` under this one. `[MOD-3]` — both import forms.
from std.math import sqrt, hypot
import std.math

fn main():
    println(sqrt(2.25))
    println(hypot(3.0, 4.0))
    println(clamp(200, 0, 100))
    println(math.lerp(0.0, 10.0, 0.25))
#$ stdout: 1.5
#$ 5.0
#$ 100
#$ 2.5
