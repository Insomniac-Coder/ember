#$ test: compile-fail
#$ rules: TYP-23
## `[TYP-23]` — a lambda parameter's word is what is written on it, never inferred, and matches the
## callable type it is given to: one that changes the caller's value says `ref mut`, as the type
## does (`fn(ref mut int)`); a plain parameter only reads.

fn inc(ref mut n: int):
    n += 1

fn bump_each(f: fn(ref mut int)):
    n = 1
    f(ref mut n)
    println(n)

fn main():
    bump_each(fn(ref mut n) => inc(ref mut n))
    bump_each(fn(n) => println(n))    #$ error[E2020]: callable parameter mode mismatch at parameter 0: expected `ref mut`, found `ref`
