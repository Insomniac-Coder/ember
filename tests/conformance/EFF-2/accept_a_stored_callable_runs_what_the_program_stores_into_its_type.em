#$ test: run-pass
#$ rules: EFF-2, EFF-5, CLO-10
#$ profiles: debug, development, release
#$ stdout: 6
# `[EFF-2]` (A4-1) — a call through a field of a callable type runs one of the functions the program
# stores into that type, a lambda with its state included: here the one lambda multiplies, and the
# `@noalloc` function calling through the field is accepted. Before, a call through a callable value
# with state had every effect.

struct Scaler:
    by: fn(int) -> int

@noalloc
fn scaled(mut s: Scaler, x: int) -> int:
    return s.by(x)

fn main():
    k = 3
    s = Scaler(by = owned fn(x: int) -> int => x * k)
    println(scaled(s, 2))
