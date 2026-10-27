#$ test: run-pass
#$ rules: THR-1, CLO-1, OWN-8
#$ stdout: 3
## design rules: R15, R12, R3
## R15 (verdict 93): "For a scoped task, `ref t` lets it look at the element `t` is on in that round,
## with no copy, as Rust's scoped threads borrow `&t`: the walked list cannot change until every task
## has ended at the end of the `with scope` block" (`scope.spawn(fn() ref t: total +=
## t.split_whitespace().count())`, R15's own line); R3: `ref t` is a read-only view of what `t` names at
## that moment.

import std.thread

fn count_words(texts: Array[String]) -> int:
    total = 0
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn() ref t: total += t.split_whitespace().count())
    return total

fn main():
    println(count_words([String("a b"), String("c")]))
