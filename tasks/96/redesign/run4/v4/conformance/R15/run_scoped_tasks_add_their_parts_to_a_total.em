#$ test: run-pass
#$ rules: R15, R12
#$ stdout: 3
## R15: "Scoped tasks need no capture words but for a loop's own variable ...: they use the real
## variables, and name on their first lines the ones they change"; the loop's own variable is given
## with `with copy of` (verdict 65); "Reductions (`total += e` ...) get one private accumulator per
## task, combined at the join"; nothing beyond C's per-thread counter runs (section 2). AN-71's
## `count_words`.

import std.thread

fn count_words(texts: Array[String]) -> int:
    total = 0
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn() with copy of t changes total: total += t.split_whitespace().count())
    return total

fn main():
    println(count_words([String("a b"), String("c")]))
