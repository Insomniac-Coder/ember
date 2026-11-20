# Clause BCK-1.1: "a call's result (...) carry the views of the arguments they may point into"
#$ test: run-pass
#$ rules: BCK-1, LT-1
#$ stdout: 2
#$ stdout: 1
# `first_of` points into its first argument only, so a push to the second ends nothing it gives.

fn first_of(a: Span[int], b: Span[int]) -> Span[int] from a:
    println(b.len())
    return a

fn main():
    xs = [1]
    ys = [1, 2]
    r = first_of(xs, ys)
    ys.push(owns 3)
    println(r[0])
