# Clause THR-21.1: "A task's statement of one of these forms on a place `P` that the task otherwise
# neither reads nor writes, `P op= e` (...), `P = max(P, e)` (...), `P.push(owns e)` (...) is a
# reduction: each task gathers its changes in a private accumulator, and the accumulators are
# combined into the place at the join, in spawn order"
#$ test: run-pass
#$ rules: THR-21, THR-11, THR-13
#$ stdout: 12
#$ stdout: 3 4 5
#$ stdout: 5
#$ stdout: 7
#$ stdout: 0.6
# A sum, pushes in spawn order, a `max`, an unsigned place taken from by two tasks (the spec's 10 -
# 1 - 2), and the spec's float sum, which gives `0.6`. Its task body, `for x in part: ftotal += x`,
# is the spec's own line; Q4 asks whether `[GRM-17]` refuses it.

import std.thread

fn main():
    total = 0
    parts: Array[int] = []
    best = 0
    with scope = thread.scope():
        for x in [3, 4, 5]:
            scope.spawn(owns fn() copy of x: total += x)
            scope.spawn(owns fn() copy of x: parts.push(owns x))
            scope.spawn(owns fn() copy of x: best = max(best, x))
    println(total)
    println(parts[0], parts[1], parts[2])
    println(best)
    hp: u8 = 10
    with scope = thread.scope():
        scope.spawn(owns fn(): hp -= 1)
        scope.spawn(owns fn(): hp -= 2)
    println(hp)
    ftotal = 0.0
    with scope = thread.scope():
        for part in [[0.1], [0.2, 0.3]]:
            scope.spawn(owns fn() copy of part: for x in part: ftotal += x)
    println(ftotal)
