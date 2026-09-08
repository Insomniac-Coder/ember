#$ test: run-pass
#$ rules: BRW-2, BRW-6, LT-1, LT-5
#$ stdout: 7
#$ stdout: 9
#$ stdout: 8

## The other side of `compile-fail/borrows_travel_with_the_reference.em`: the
## same three shapes, with each borrow finished before the owner is touched
## again. `[BRW-2]` is what makes them legal — a borrow lasts until its last
## use, not to the end of the scope — and a region that ends where the last
## reference derived from it dies is what says so.

## The copy is the last holder, and it is done before `n` is read.
fn through_a_copy() -> i32:
    n: i32 = 1
    r: ref mut i32 = ref mut n
    s: ref mut i32 = r
    s = 7
    return n

## `[BRW-6]` — the reborrow ends, and with it the borrow it derived from.
fn through_a_reborrow() -> i32:
    m: i32 = 1
    r: ref mut i32 = ref mut m
    q: ref mut i32 = ref mut r
    q = 9
    return m

## `[LT-1]` rule 2: the result points into the one view-typed parameter, so
## the caller holds the borrow only while it holds the result.
fn keep(a: ref i32) -> ref i32:
    return a

fn through_a_call() -> i32:
    k: i32 = 3
    r: ref i32 = keep(ref k)
    v: i32 = r
    k = 5
    return v + k

fn main():
    println(through_a_copy())
    println(through_a_reborrow())
    println(through_a_call())
