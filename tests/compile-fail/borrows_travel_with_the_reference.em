#$ test: compile-fail
#$ rules: BRW-1, BRW-2, BRW-6, LT-1, LT-5, DIA-3
#$ error[E3021]: `n` cannot be written while it is borrowed
#$ error[E3021]: `m` cannot be written while it is borrowed
#$ error[E3021]: `k` cannot be written while it is borrowed

## `[LT-5]` — a borrow's region is the set of points where it must be valid,
## and that is not the same as the liveness of the local it was first written
## into. Each function here ends the first local's life and keeps the borrow
## alive through something else: a copy, a reborrow, and a call that hands the
## reference back.
##
## Before regions existed all three compiled in silence, which is a dangling
## write in safe code.

## The reference is copied, and the copy is what keeps the loan alive.
fn through_a_copy() -> i32:
    n: i32 = 1
    r: ref mut i32 = ref mut n
    s: ref mut i32 = r
    n = 5
    s = 7
    return n

## `[BRW-6]` — a reborrow derives from the reference it goes through, so the
## first loan outlives the local that took it.
fn through_a_reborrow() -> i32:
    m: i32 = 1
    r: ref mut i32 = ref mut m
    q: ref mut i32 = ref mut r
    m = 5
    q = 7
    return m

## `[LT-1]` at the call site: `keep` returns a view, and the only view-typed
## parameter is what it can point into, so the caller holds the borrow for as
## long as it holds the result.
fn keep(a: ref i32) -> ref i32:
    return a

fn through_a_call() -> i32:
    k: i32 = 1
    r: ref i32 = keep(ref k)
    k = 5
    return r

fn main():
    println(through_a_copy())
    println(through_a_reborrow())
    println(through_a_call())
