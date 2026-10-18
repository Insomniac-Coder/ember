#$ test: run-pass
#$ rules: DSJ-7, UNS-1
#$ profiles: debug, release, shipping
#$ stdout: 5

# `[DSJ-7]` — `assume_disjoint` gives the pair without comparing; the caller
# answers for it, inside `unsafe`.

from std.mem import assume_disjoint

fn main():
    x: Array[int] = [1, 2, 3]
    y: Array[int] = [4]
    pair: (Span[int], Span[int])
    unsafe:  # SAFETY: `x` and `y` are two arrays, so their views cannot overlap
        pair = assume_disjoint(x[..1], y[..])
    (u, v) = pair
    println(u[0] + v[0])
