#$ test: run-fail
#$ rules: CG-C-1, TYP-8
#$ profiles: debug, development
#$ panics: overflow
# `[CG-C-1]` — `MIN // -1` is handled before the C operator runs: it is the language's overflow
# panic, never the C operator's undefined behaviour.

fn main():
    xs: Array[int] = [-9223372036854775807 - 1, -1]
    println(xs[0] // xs[1])
