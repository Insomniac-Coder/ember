#$ test: compile-fail
#$ rules: GRM-4
# `[GRM-4]` — redeclaring a name in the same block is `E1020` (D-240: the
# second declaration silently shadowed the first).

fn main():
    z: int = 1
    z: int = 2              #$ error[E1020]: `z` is already declared in this block
    println(z)
