#$ test: run-pass
#$ rules: EFF-17
#$ stdout: 1
# `[EFF-17]` — the spelling with its argument.

@nopanic(explicit)
fn f(n: int) -> int:
    return n

fn main():
    println(f(1))
