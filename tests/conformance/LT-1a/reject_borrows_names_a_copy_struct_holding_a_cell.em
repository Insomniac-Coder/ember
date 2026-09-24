#$ test: compile-fail
#$ rules: LT-1a, BRW-8
# ODR-024 — a borrowed `Copy` parameter is not a source, even one holding a
# `Cell`, which `[BRW-8]` passes by address.

@derive(Copy)
struct Counter:
    hits: Cell[int]

@borrows(c)    #$ error[E2031]: `@borrows` names `c`, which a result cannot borrow
fn peek(c: Counter) -> ref Cell[int]:
    return ref c.hits

fn main():
    println(0)
