#$ test: run-pass
#$ rules: TYP-14, BRW-1, BRW-2, BRW-6, EXP-5
#$ stdout: 10
#$ stdout: 10
#$ stdout: 11
#$ stdout: 7

## `ref place` and `ref mut place` are the explicit borrow forms. They are
## needed only when initialising a `ref`-typed local or a view struct's field:
## every other borrow is implicit in a parameter mode, which is why a call site
## never writes one.
##
## Each borrow below is dead before the next access to the owner, which is what
## `[BRW-2]` asks and what a scope-based rule could not express. An earlier
## draft of this file kept a reborrow at the end, which held the first borrow
## live across everything between it and the top — and the borrow checker said
## so.

fn bump(mut n: i32):
    n = n + 1

## `[TYP-14]` — writing a `ref mut` local writes through to the referent.
fn write_through() -> i32:
    n: i32 = 1
    r: ref mut i32 = ref mut n
    r = 10
    return n

## A shared reference reads through the same way.
fn read_through() -> i32:
    n: i32 = 10
    s: ref i32 = ref n
    return s

## `[BRW-6]` — `ref mut` of a `ref mut` local reborrows rather than copying the
## reference, because the operand is already a deref.
fn reborrow() -> i32:
    n: i32 = 1
    r: ref mut i32 = ref mut n
    q: ref mut i32 = ref mut r
    q = 7
    return n

fn main():
    println(write_through())
    println(read_through())

    n: i32 = 10
    bump(n)
    println(n)

    println(reborrow())
