#$ test: run-pass
#$ rules: OWN-9, OWN-8
#$ stdout: 5
#$ 5
# `[OWN-9]` — `take(copy of n)` gives `take` a copy and keeps the caller's own.

fn take(owns x: int):
    println(x)

fn main():
    n = 5
    take(copy of n)
    println(n)
