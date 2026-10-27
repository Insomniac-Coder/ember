#$ test: run-pass
#$ rules: R15
#$ stdout: 0.6
## R15 (verdict 92): "a decimal total may differ from [adding one by one] in the last digit
## (`for part in [[0.1], [0.2, 0.3]]:` with `scope.spawn(fn() copy of part: for x in part: total += x)`
## gives `0.6`, where adding one by one gives `0.6000000000000001`) and is the same on every run, as
## Rust's Rayon and C's OpenMP give": each task keeps its own total (0.1 and 0.5), added at the join in
## spawn order. It replaces step 3's `reject_a_float_sum_added_more_than_once_per_task.em` (E7007, AN-71's
## refused `total_of`), a refusal verdict 92 removed.

import std.thread

fn total_of(parts: Array[Array[float]]) -> float:
    total = 0.0
    with scope = thread.scope():
        for part in parts:
            scope.spawn(fn() copy of part: for x in part: total += x)
    return total

fn main():
    println(total_of([[0.1], [0.2, 0.3]]))
