#$ test: run-pass
#$ rules: OWN-9, CTL-3
#$ stdout: 0
#$ 1
#$ 2
#$ [0, 1, 2]
# `[OWN-9]`, `[CTL-3]` — a counted loop's name is a new value each turn, so a turn may hand it over:
# the next turn has its own.

fn take(owns x: int):
    println(x)

fn main():
    for i in 0..3:
        take(i)
    xs: Array[int] = []
    for j in range(3):
        xs.push(j)
    println(xs)
