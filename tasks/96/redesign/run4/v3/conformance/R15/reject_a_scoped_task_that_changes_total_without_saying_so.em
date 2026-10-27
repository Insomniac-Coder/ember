#$ test: compile-fail
#$ rules: R15, R12
## R15: scoped tasks "name on their first lines the ones they change" (verdict 22; AN-71's OLD task
## line).

import std.thread

fn count_words(texts: Array[String]) -> int:
    total = 0
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn(): total += t.split_whitespace().count())    #$ error[E2105]: this function changes `total`, a variable it uses from `count_words`; say so on its first line: `scope.spawn(fn() changes total: total += t.split_whitespace().count())`
    return total

fn main():
    println(count_words([String("a b"), String("c")]))
