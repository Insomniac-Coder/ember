#$ test: run-pass
#$ rules: EFF-21
#$ stdout: 3
# `[EFF-21]` — a function that takes no lock keeps `@nolock`.

@nolock
fn add(a: int, b: int) -> int:
    return a + b

fn main():
    println(add(1, 2))
