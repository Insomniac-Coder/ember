#$ test: compile-fail
#$ rules: R15, R12
## R12 (verdict 29): a change through an `Atomic` is named like every change; a captured variable by
## its name (verdict 22; AN-73's refused v2 line).

import std.thread
from std.sync import Atomic

fn count(texts: Array[String]) -> int:
    total = Atomic(0)
    with scope = thread.scope():
        for t in texts:
            n = t.split_whitespace().count()
            scope.spawn(fn() with copy of n: total.fetch_add(n))    #$ error[E2105]: this function changes `total` through `fetch_add`, a variable it uses from `count`; say so on its first line: `scope.spawn(fn() with copy of n changes total: total.fetch_add(n))`
    return total.load()

fn main():
    println(count([String("a b"), String("c")]))
