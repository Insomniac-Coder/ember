#$ test: compile-fail
#$ rules: R15
## R15: reductions are combined at the join; "nothing reads the place before the join" (E7007; AN-71).

import std.thread

fn count_words(texts: Array[String]) -> int:
    total = 0
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn() changes total: total += t.split_whitespace().count())
        scope.spawn(fn() => println(total))    #$ error[E7007]: tasks add their parts to `total`, which is known only once the `with` block (lines 9 to 12) ends; read `total` after it, as line 13 does
    return total

fn main():
    println(count_words([String("a b"), String("c")]))
