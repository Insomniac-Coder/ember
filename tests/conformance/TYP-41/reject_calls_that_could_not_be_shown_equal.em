#$ test: compile-fail
#$ rules: TYP-41
#$ error[E2020]: expected `[i64; cube(N)]`, found `[i64; square(N)]`, and `cube(N)` and `square(N)` could not be shown equal
# `[TYP-41]` rule 3 — two calls are one term only when they call the same function on equal
# arguments: `square(N)` and `cube(N)` are different types, whatever an optimiser could prove.

fn square(x: int) -> int:
    return x * x

fn cube(x: int) -> int:
    return x * x * x

fn reshape[const N: int](g: [int; square(N)]) -> [int; cube(N)]:
    return g

fn main():
    println(len(reshape[1]([0])))
