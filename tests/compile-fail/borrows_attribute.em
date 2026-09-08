#$ test: compile-fail
#$ rules: LT-1a, LT-1
#$ error[E2031]: `@borrows` names `n`, which is not view-typed
#$ error[E2031]: `@borrows` names `missing`, which is not a parameter
#$ error[E2031]: `@borrows` is only meaningful on a function that returns a view

## `[LT-1a]` — `@borrows(p, …)` overrides the region `[LT-1]`'s elision would
## give a view-typed return, so the caller may keep using the parameters it
## does not name. It is part of the function's public contract: `[VER-2]` makes
## widening it a breaking change, so a typo in it is worth catching loudly.

@borrows(n)
fn names_a_value(a: ref i32, n: i32) -> ref i32:
    return a

@borrows(missing)
fn names_nothing(a: ref i32) -> ref i32:
    return a

@borrows(a)
fn returns_a_value(a: ref i32) -> i32:
    return a

fn main():
    println(1)
