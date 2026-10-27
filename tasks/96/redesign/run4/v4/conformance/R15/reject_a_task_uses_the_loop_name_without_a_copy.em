#$ test: compile-fail
#$ rules: R15, R12
## R15 (verdict 65): "The loop's own variable, which changes every round, ... [is] given to a task or a
## thread with `with copy of`, since it may still run in a later round"; for a scoped task the code is
## the one a local of one round gets (E3026, AN-73).

import std.thread

fn count_words(texts: Array[String]) -> int:
    total = 0
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn() changes total: total += t.split_whitespace().count())    #$ error[E3026]: the task uses `t`, the loop's name (line 12), which names the next text in the next round while the task may still be running; give the task its own copy: `scope.spawn(fn() with copy of t changes total: total += t.split_whitespace().count())`
    return total

fn main():
    println(count_words([String("a b"), String("c")]))
