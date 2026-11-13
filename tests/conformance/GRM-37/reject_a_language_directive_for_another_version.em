#$ test: compile-fail
#$ rules: GRM-37, VER-8
#! language "0.9.8"    #$ error[E0006]: language version `0.9.8` is not the current language
## `[GRM-37]` — `#! language "<version>"` is accepted only for the current version (`[VER-8]`, `E0006`).

fn main():
    println(1)
