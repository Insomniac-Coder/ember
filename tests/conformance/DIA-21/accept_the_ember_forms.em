#$ test: run-pass
#$ rules: DIA-21
#$ profiles: debug, release, shipping
#$ stdout: [2, 3] [1, 2] [3]
# `[DIA-21]` — the Ember forms the Python habits are answered with compile.

fn main():
    xs = [1, 2, 3]
    println(xs[1..3], xs[..2], xs[2..])
