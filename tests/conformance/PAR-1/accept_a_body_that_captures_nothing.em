#$ test: run-pass
#$ rules: PAR-1, PAR-3
#$ profiles: debug, release
#$ stdout: 1998000
# `[PAR-1]` — a body that captures nothing (its only name of the enclosing code a reduction, which
# each chunk has its own copy of) is a plain function over a chunk, with no environment.

fn main():
    total = 0
    @parallel(chunk=100, reduce=[total: +])
    for i in 0..4000:
        total += i % 1000
    println(total)
