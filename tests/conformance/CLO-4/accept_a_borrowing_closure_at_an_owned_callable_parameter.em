#$ test: run-pass
#$ rules: CLO-1, CLO-2, CLO-3, CLO-4, CLO-15, TYP-15
#$ profiles: debug, release
#$ stdout: 7
#$ stdout: 9
# `[CLO-4]`, `[CLO-15]` (ODR-100) — an ordinary closure may be passed to an
# `owned` callable parameter: the callee holds a view, and whatever it returns
# holding the closure lives no longer than the closure's captures. Until
# ODR-100 this was refused at the call (`E3063`), whatever the callee did.

fn invoke(owned f: fn() -> i32) -> i32:
    return f()

fn keep[F: fn() -> i32](owned f: F) -> F:
    return f

fn main():
    value: i32 = 7
    closure = fn() => value
    println(invoke(closure))
    other: i32 = 9
    held = keep(fn() => other)
    println(held())
