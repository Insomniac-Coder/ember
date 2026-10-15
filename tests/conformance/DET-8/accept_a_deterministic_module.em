#! module deterministic
#$ test: run-pass
#$ rules: DET-8
#$ stdout: 6
# `[DET-8]` — a module whose functions are all `Nondet`-free.

fn triple(x: int) -> int:
    return x * 3

fn main():
    println(triple(2))
