#$ test: compile-fail
#$ rules: EFF-17
#$ error[E0104]: `@nopanic` is reserved for a later version
#$ help: write `@nopanic(explicit)`
# `[EFF-17]` — `@nopanic(explicit)` is spelled with its argument; bare `@nopanic` is `E0104`, with
# the fix, so that no reader takes it to mean "cannot abort".

@nopanic
fn f(n: int) -> int:
    return n

fn main():
    println(f(1))
