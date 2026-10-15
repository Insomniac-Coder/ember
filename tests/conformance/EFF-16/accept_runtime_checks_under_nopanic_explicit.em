#$ test: run-pass
#$ rules: EFF-16
#$ stdout: 4
# `[EFF-16]` — `@nopanic(explicit)` forbids only the panics the programmer writes and allocation
# failure: a bounds check and an overflow check are `RuntimeCheck`s, allowed under it.

@nopanic(explicit)
fn pick(xs: Span[int], i: int) -> int:
    return xs[i] + 1

fn main():
    values = [1, 2, 3]
    println(pick(values.as_span(), 2))
