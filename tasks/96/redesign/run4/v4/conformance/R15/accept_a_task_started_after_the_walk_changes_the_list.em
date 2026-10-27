#$ test: run-pass
#$ rules: R15, R8
#$ stdout: a
#$ stdout: 2
## R15: the line E3020 gives: the task that pushes to `names` is started after the walk, inside the
## scope; nothing in the parent reads `names` before the join.

import std.thread

fn grow(ref mut names: Array[String]):
    with scope = thread.scope():
        for n in names:
            println(n)
        scope.spawn(fn() changes names: names.push(String("x")))

fn main():
    names = [String("a")]
    grow(ref mut names)
    println(names.len())
