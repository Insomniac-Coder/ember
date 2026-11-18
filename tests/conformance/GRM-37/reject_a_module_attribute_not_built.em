#$ test: compile-fail
#$ rules: GRM-37, ATT-6
## `[GRM-37]` — a valid module attribute whose effect is not built is `E0900` (`[ATT-6]`), never silently
## ignored.
#! module prelude    #$ error[E0900]: `@prelude` is not implemented yet

fn main():
    println(1)
