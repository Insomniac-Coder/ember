#$ test: run-pass
#$ rules: R15
#$ stdout: 1.75
## R15: one add per task, combined at the join in spawn order (AN-71); a reduction of decimal numbers,
## as `run_a_float_sum_added_more_than_once_per_task_is_a_reduction.em` is with several adds per task
## (verdict 92).

import std.thread

fn total_of(parts: Array[Array[float]]) -> float:
    sum = 0.0
    with scope = thread.scope():
        for p in parts:
            scope.spawn(fn() copy of p: sum += p.iter().sum())
    return sum

fn main():
    println(total_of([[0.5, 0.25], [1.0]]))
