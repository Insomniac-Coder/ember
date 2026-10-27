#$ test: compile-fail
#$ rules: R15
## R15: reductions are combined at the join; "nothing reads the place before the join" (E7007; AN-71;
## the message gives the line to remove and the line to write, R18). The nearest allowed program is
## `run_scoped_tasks_add_their_parts_to_a_total.em`, which reads `total` after the block.

import std.thread

fn count_words(texts: Array[String]) -> int:
    total = 0
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn() copy of t: total += t.split_whitespace().count())
        scope.spawn(fn() => println(total))    #$ error[E7007]: tasks add their parts to `total`, which is known only once the `with` block (lines 11 to 14) ends; read it after the block: remove line 14 and write `println(total)` above line 15
    return total

fn main():
    println(count_words([String("a b"), String("c")]))
