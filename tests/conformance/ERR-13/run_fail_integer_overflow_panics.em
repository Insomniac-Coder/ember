#$ test: run-fail
#$ rules: ERR-13
#$ panics: integer overflow
# `[ERR-13]` — integer overflow is the program's own arithmetic failing: it panics.

fn main():
    a = 9223372036854775807
    b = 1
    println(a + b)
