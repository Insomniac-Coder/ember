#$ test: run-pass
#$ rules: CG-C-3a, ATT-6
#$ profiles: debug, release, shipping
#$ stdout: 15 4 -1 7
# `[CG-C-3a]` — `@inline` is binding and `@noinline`, `@cold` and `@hot` are
# hints to the C compiler; none changes what the program computes.

@inline
fn twice(x: int) -> int:
    return x * 2

@noinline
fn thrice(x: int) -> int:
    return x * 3

@cold
fn failed(code: int) -> int:
    return -code

@hot
fn step(x: int) -> int:
    return x + 1

fn main():
    total = 0
    for i in 0..3:
        total += twice(i) + thrice(i)
    println(total, step(3), failed(1), twice(3) + step(0))
