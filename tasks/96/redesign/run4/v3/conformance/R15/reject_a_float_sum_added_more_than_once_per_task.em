#$ test: compile-fail
#$ rules: R15
## R15: reductions are combined at the join in spawn order; a float place one task adds to more than
## once would not give any order of single adds (E7007, AN-71's refused `total_of`).

import std.thread

fn total_of(parts: Array[Array[float]]) -> float:
    sum = 0.0
    with scope = thread.scope():
        for p in parts:
            scope.spawn(fn() changes sum: for x in p: sum += x)    #$ error[E7007]: each task adds to `sum` once per number (line 12); float sums depend on how the adds are grouped, so the result would not be any order of single adds; add up each part in its task and add it to `sum` once: `scope.spawn(fn() changes sum: sum += p.iter().sum())` instead of line 12
    return sum

fn main():
    println(total_of([[0.5, 0.25], [1.0]]))
