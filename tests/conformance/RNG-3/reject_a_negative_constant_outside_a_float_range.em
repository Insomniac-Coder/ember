#$ test: compile-fail
#$ rules: RNG-3, RNG-10
#$ error[E2211]: -1.5 is outside `Unit`
# D-435 — a negated constant outside the range is the constant diagnostic,
# `E2211`, not `E2215`'s "not known to be in range".

type Unit = f64 in -1.0 ..= 1.0

fn main():
    a: Unit = -1.5
    println(a)
