#$ test: run-pass
#$ rules: R15
#$ stdout: 1.75
## R15: one add per task: "the spawn-order result is one order of single adds" (AN-71, the line E7007
## gives).

import std.thread

fn total_of(parts: Array[Array[float]]) -> float:
    sum = 0.0
    with scope = thread.scope():
        for p in parts:
            scope.spawn(fn() with copy of p changes sum: sum += p.iter().sum())
    return sum

fn main():
    println(total_of([[0.5, 0.25], [1.0]]))
