#$ test: run-pass
#$ rules: TYP-23
#$ stdout: 2
# `[TYP-23]` — a parameter that changes the caller's value says `ref mut` (`fn(ref mut n)`), as
# the type does (`fn(ref mut int)`).

fn inc(ref mut n: int):
    n += 1

fn bump_each(f: fn(ref mut int)):
    n = 1
    f(ref mut n)
    println(n)

fn main():
    bump_each(fn(ref mut n) => inc(ref mut n))
