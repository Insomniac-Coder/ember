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
    unsafe:
        (u, v) = assume_disjoint(x[..1], y[..])
        println(u[0] + v[0])
