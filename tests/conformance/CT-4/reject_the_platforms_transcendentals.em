#$ test: compile-fail
#$ rules: CT-4, DET-2
#$ error[E6010]: this `comptime(…)` expression cannot run while compiling: it reaches a transcendental function from the platform library
# `[CT-4]` — compile time has no source of `Nondet`: the platform library's `sin` does not run
# there, and `det.sin` is never put in its place.

import math

fn main():
    println(comptime(math.sin(1.0)))
