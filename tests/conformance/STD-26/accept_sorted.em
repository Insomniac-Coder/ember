#$ test: run-pass
#$ rules: STD-26
#$ profiles: debug, release, shipping
#$ stdout: 1 3 3 7 9 -2.0
# `sorted(xs)` returns a new sorted array and leaves `xs` as it was.

fn main():
    xs = [3, 1, 2]
    ys = sorted(xs)
    fixed: [int; 3] = [9, 7, 8]
    zs = sorted(fixed)
    ws = sorted([0.5, -2.0])
    println(ys[0], ys[2], xs[0], zs[0], zs[2], ws[0])
