#$ test: run-pass
#$ rules: THR-1
#$ stdout: 3
## design rules: R15
## R15 (verdict 77): reductions "on a local or on a field of an object the tasks reach
## (`scope.spawn(fn() copy of x: stats.total += x)`) ... get one private accumulator per task, combined
## at the join in spawn order"; a whole-number total is always the one adding one by one gives; v3
## refused this program (E7007).

import std.thread

class Stats:
    total: int = 0

fn count(texts: Array[String]) -> int:
    stats = Stats()
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn() copy of t: stats.total += t.len())
    return stats.total

fn main():
    println(count([String("ab"), String("c")]))
