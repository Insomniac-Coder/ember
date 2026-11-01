#$ test: run-pass
#$ rules: CLO-1, CLO-2, CLO-3, CLO-4, CLO-15, TYP-15
#$ profiles: debug, development
#$ stdout: 7
#$ stdout: 9
# `[CLO-4]`, `[CLO-15]` (ODR-100) — an ordinary closure may be passed to an
# `owned` callable parameter: the callee holds a view, and whatever it returns
# holding the closure lives no longer than the closure's captures. Until
# ODR-100 this was refused at the call (`E3063`), whatever the callee did.

fn invoke(owns f: fn() -> i32) -> i32:
    return f()

[F: needs fn() -> i32]
fn keep(owns f: F) -> F:
    return f

fn main():
    value: i32 = 7
    closure = fn() => value
    println(invoke(owns closure))
    other: i32 = 9
    held = keep(owns fn() => other)
    println(held())
