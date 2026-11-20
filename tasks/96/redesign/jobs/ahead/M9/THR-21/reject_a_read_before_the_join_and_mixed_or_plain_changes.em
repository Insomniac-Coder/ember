# Clause THR-21.2: "All the tasks' statements on one place use one operator: `+=` in one task and
# `*=` in another (...) is `E7007`"; "Nothing reads the place before the join (`E7007`"; "A plain
# assignment from a task (`a.target.hp = 5`) and a task that reads the current value (...) need a
# lock (`E7007`)."
#$ test: compile-fail
#$ rules: THR-21, THR-11
#$ error[E7007]
#$ error[E7007]
# The read before the join has the spec's message, with this file's line numbers; the mixed
# operators and two plain assignments are claimed for the file (Q3).

import std.thread

fn main():
    total = 0
    with scope = thread.scope():
        for x in [1, 2]:
            scope.spawn(owns fn() copy of x: total += x)
        println(total)    #$ error[E7007]: tasks add their parts to `total`, which is known only once the `with` block (lines 16 to 19) ends; read it after the block: remove line 19 and write `println(total)` above line 20
    println("done")
    prod = 1
    with scope = thread.scope():
        scope.spawn(owns fn(): prod += 2)
        scope.spawn(owns fn(): prod *= 3)
    hp = 10
    with scope = thread.scope():
        scope.spawn(owns fn(): hp = 5)
        scope.spawn(owns fn(): hp = 6)
    println(prod, hp)
