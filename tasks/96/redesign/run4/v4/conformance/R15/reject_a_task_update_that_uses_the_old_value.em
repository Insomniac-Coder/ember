#$ test: compile-fail
#$ rules: R15
#$ help: to add each task's part, write `total += t.len()`; to combine the parts in order, write `parts: Array[int] = []` before the `with` block, `scope.spawn(fn() with copy of t changes parts: parts.push(t.len()))` as the task, and `for p in parts: total = total * 2 + p` after the `with` block
## R15: a change that is not one of the reductions, by two tasks, is E7007; the conflict is reported once,
## on the captured name that causes it (AN-72, with the loop variable's copy of verdict 65).

import std.thread

fn count_words(texts: Array[String]) -> int:
    total = 0
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn() with copy of t changes total: total = total * 2 + t.len())    #$ error[E7007]: two tasks change `total`, and `total = total * 2 + t.len()` uses its old value
    return total

fn main():
    println(count_words([String("ab"), String("c")]))
