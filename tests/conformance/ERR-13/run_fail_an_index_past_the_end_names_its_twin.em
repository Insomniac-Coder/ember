#$ test: run-fail
#$ rules: ERR-13
#$ panics: index 7 is out of bounds for a length of 3; use .get(i) for an Option
# `[ERR-13]` — an operation whose failure is a bug in the caller panics, and the panic message names
# the non-panicking twin: `xs.get(i)` returns an `Option`.

fn main():
    xs = [1, 2, 3]
    i = 7
    println(xs[i])
