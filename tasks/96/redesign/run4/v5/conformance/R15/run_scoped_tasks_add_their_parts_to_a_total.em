#$ test: run-pass
#$ rules: THR-1, CLO-1
#$ stdout: 3
## design rules: R15, R12
## R15: "Scoped tasks need no capture words but for a loop's own variable ...: they use the real
## variables"; what a task changes is worked out from its body (verdict 86); the loop's own variable is
## given with `copy of` (verdicts 65, 83); "Reductions (`total += e` ...) get one private accumulator per
## task, combined at the join"; nothing beyond C's per-thread counter runs (section 2). AN-71's
## `count_words`.

import std.thread

fn count_words(texts: Array[String]) -> int:
    total = 0
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn() copy of t: total += t.split_whitespace().count())
    return total

fn main():
    println(count_words([String("a b"), String("c")]))
