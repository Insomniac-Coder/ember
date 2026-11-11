#$ test: run-pass
#$ rules: GRM-20
#$ stdout: 6
# `[GRM-20]` — a statement carries no attribute but the four; one with none is plain code.

fn main():
    total = 0
    for i in 0..4:
        total += i
    println(total)
