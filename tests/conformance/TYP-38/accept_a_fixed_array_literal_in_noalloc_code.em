#$ test: run-pass
#$ rules: TYP-38
#$ stdout: 3
# `[TYP-38]` — a literal in a fixed-array position makes no allocation, so `@noalloc` holds.

@noalloc
fn fixed(n: int) -> int:
    xs: [int; 3] = [n, n, n]
    return xs[0] + xs[1] + xs[2]

fn main():
    println(fixed(1))
