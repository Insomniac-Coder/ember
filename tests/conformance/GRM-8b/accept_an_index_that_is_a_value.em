#$ test: run-pass
#$ rules: GRM-8b
#$ stdout: 20
# `[GRM-8b]` — an index whose argument is a value is an index.

fn main():
    xs = [10, 20, 30]
    println(xs[1])
