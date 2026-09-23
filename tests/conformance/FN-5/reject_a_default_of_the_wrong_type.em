#$ test: compile-fail
#$ rules: FN-5
#$ error[E2020]: expected `i64`, found `str`
# `[FN-5]` — a default is checked once, where it is declared, however many
# calls use it.

fn bad(x: int = "text") -> int:
    return x

fn main():
    println(bad(), bad())
