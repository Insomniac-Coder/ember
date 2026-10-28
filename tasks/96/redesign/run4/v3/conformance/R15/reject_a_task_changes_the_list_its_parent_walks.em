#$ test: compile-fail
#$ rules: R15, R8
## R15: "a scoped task's change to what its parent walks is E7007"; every task's effects are checked
## against the parent until the join. R8 (c) would call the same program E3020 ("a `spawn` whose thread
## changes it"); which one is open (see the matrix, R15-6). Message written to R18.

import std.thread

fn grow(ref mut xs: Array[int]):
    with scope = thread.scope():
        for x in xs:
            scope.spawn(fn() changes xs: xs.push(x))    #$ error[E7007]: the task started on line 12 adds to `xs`, which this loop walks (line 11) until the join; collect what to add in a list of your own: write `more: Array[int] = []` before the `with` block, `more.push(x)` in place of line 12, and `xs.extend(more)` after the `with` block

fn main():
    xs = [1, 2]
    grow(ref mut xs)
    println(xs.len())
