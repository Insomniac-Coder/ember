#$ test: run-pass
#$ rules: THR-5, THR-11
#$ profiles: debug, release
#$ stdout: 7 1
#$ 3
#$ 4
# `[THR-5]` — on every exit from a scope's `with` block (its end, `return`, `break`, `continue`)
# its threads are joined before the exit completes: a `return`'s value is computed first.

import std.thread
from std.sync import Atomic

fn bump(count: Atomic[int]):
    count.fetch_add(1)

fn early(count: Atomic[int]) -> int:
    with scope = thread.scope():
        scope.spawn(fn() => bump(count))
        return 7

fn in_loop(count: Atomic[int]):
    for i in range(3):
        with scope = thread.scope():
            scope.spawn(fn() => bump(count))
            if i == 1:
                break
            continue

fn main():
    count = Atomic(0)
    println(early(count), count.load())
    in_loop(count)
    println(count.load())
    with scope = thread.scope():
        scope.spawn(fn() => bump(count))
    println(count.load())
