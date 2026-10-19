#$ test: run-pass
#$ rules: PAR-3, PAR-1
#$ profiles: debug, development
#$ stdout: 49995000 9999 0 true
#$ stdout: 0.30000000000000004
# `[PAR-3]` — each reduction has a copy per chunk, starting at its operator's identity, and the
# chunks' copies are folded into the variable in chunk order after the loop: the result is the
# one a serial loop would give, whichever thread ran which chunk.

fn main():
    n = 10000
    values: Array[int] = []
    for i in range(n):
        values.push(i)
    total = 0
    best = 0
    least = 5
    all_small = true
    @parallel(reduce=[total: +, best: max, least: min, all_small: and])
    for i in 0..n:
        total += values[i]
        best = max(best, values[i])
        least = min(least, values[i])
        all_small = all_small and values[i] < n
    println(f"{total} {best} {least} {all_small}")
    parts = [0.1, 0.2]
    sum = 0.0
    @parallel(reduce=[sum: +])
    for i in 0..2:
        sum += parts[i]
    println(sum)
