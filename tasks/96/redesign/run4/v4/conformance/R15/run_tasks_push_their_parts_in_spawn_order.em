#$ test: run-pass
#$ rules: R15
#$ stdout: 2
#$ stdout: 1
#$ stdout: 5
## R15: "`push` ... get one private accumulator per task, combined at the join in spawn order"; the
## program E7007's help leads to (AN-72), which gives what one thread would.

import std.thread

fn count_words(texts: Array[String]) -> int:
    total = 0
    parts: Array[int] = []
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn() with copy of t changes parts: parts.push(t.len()))
    for p in parts:
        println(p)
        total = total * 2 + p
    return total

fn main():
    println(count_words([String("ab"), String("c")]))
