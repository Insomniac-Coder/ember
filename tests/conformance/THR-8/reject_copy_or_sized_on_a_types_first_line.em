#$ test: compile-fail
#$ rules: TYP-43, THR-8
# `[TYP-43]` (ODR-111) — `Copy` comes from `@derive(Copy)`, and `Sized` is never written: each written
# by hand is `E2040`, on a type's own first line as in an `extend`.

struct P implements Copy:    #$ error[E2040]: `Copy` comes from `@derive(Copy)`
    x: int

struct Q implements Sized:    #$ error[E2040]: `Sized` is never written
    y: int

fn main():
    println(P(x=1).x, Q(y=2).y)
