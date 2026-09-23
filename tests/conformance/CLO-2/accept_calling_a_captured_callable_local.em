#$ test: run-pass
#$ rules: CLO-2, CLO-3
#$ profiles: debug, release, shipping
#$ stdout: 2
#$ 42
# D-185 — a callable local of the enclosing function, called inside a closure,
# is a capture like any other name: by shared borrow in a plain closure, by
# copy in an `owned fn`. The call path used to report E1010.

fn inc(x: i32) -> i32:
    return x + 1

fn main():
    f: fn(i32) -> i32 = inc
    borrowed = fn() -> i32:
        return f(1)
    println(borrowed())
    owning = owned fn() => f(41)
    println(owning())
