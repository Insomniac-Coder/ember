#$ test: run-pass
#$ rules: MOD-1, MOD-3, STD-1

# `[MOD-1]` — `std` is a package, so `std.math` is `math.em` under its own
# `src/`, not `std/math.em` under this one. `[MOD-3]` — both import forms.
from std.math import clamp_f32, min_i32
import std.math

fn main():
    println(clamp_f32(2.5, 0.0, 1.0))
    println(min_i32(3, 7))
    println(math.clamp_i32(200, 0, 100))
    println(math.lerp_f32(0.0, 10.0, 0.25))
#$ stdout: 1
#$ 3
#$ 100
#$ 2.5
