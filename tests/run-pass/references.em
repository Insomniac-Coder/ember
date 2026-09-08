#$ test: run-pass
#$ rules: TYP-14, BRW-3, BRW-6, EXP-5
#$ stdout: 10
#$ stdout: 10
#$ stdout: 11
#$ stdout: 7

## `ref place` and `ref mut place` are the explicit borrow forms. They are
## needed only when initialising a `ref`-typed local or a view struct's field:
## every other borrow is implicit in a parameter mode, which is why a call site
## never writes one.

fn bump(mut n: i32):
    n = n + 1

fn main():
    n: i32 = 1
    r: ref mut i32 = ref mut n
    ## `[TYP-14]` — writing a `ref mut` local writes through to the referent.
    r = 10
    println(n)

    ## A shared reference reads through the same way.
    s: ref i32 = ref n
    println(s)

    bump(n)
    println(n)

    ## `[BRW-6]` — `ref mut` of a `ref mut` local reborrows rather than
    ## copying the reference, because the operand is already a deref.
    q: ref mut i32 = ref mut r
    q = 7
    println(n)
