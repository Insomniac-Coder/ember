#$ test: run-pass
#$ rules: PRF-3
#$ profiles: release, shipping
#$ stdout: 0
# `debug_assert` is checked only in `debug`: elsewhere its arguments are
# type-checked and not evaluated.

fn loud() -> bool:
    println("evaluated")
    return false

fn main():
    debug_assert(loud())
    println(0)
