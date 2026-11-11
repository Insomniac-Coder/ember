#$ test: run-pass
#$ rules: EFF-20
#$ stdout: 7
# `[EFF-20]` — a `@noio` function that does no I/O is accepted; the caller outside it may print.

@noio
fn double(n: int) -> int:
    return n * 2

fn main():
    println(double(3) + 1)
