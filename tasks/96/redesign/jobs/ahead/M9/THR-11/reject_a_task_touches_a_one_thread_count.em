# Clause THR-11.3: "**A task touches no one-thread count.** (...) a task that may end a `shares`
# owner, or may end an object of a class family that carries a weak count (`[OBJ-1]`), is `E7015`
# at the spawn"
#$ test: compile-fail
#$ rules: THR-11, THR-2, RC-4, OBJ-1
# The spec's two messages, with this file's line numbers.

import std.thread

class Player:
    hp: int = 5

fn main():
    a: Array[shares Player] = []
    a.push(shares Player())
    with scope = thread.scope():
        scope.spawn(owns fn(): a.clear())    #$ error[E7015]: the task started on line 17 clears `a`, which holds `shares Player` owners, whose count is kept with plain adds on one thread; clear it after the join: move `a.clear()` below the `with scope` block
    q = Player()
    w = weakref q
    with scope = thread.scope():
        scope.spawn(owns fn(): q = Player())    #$ error[E7015]: the task started on line 21 may end the `Player` that `w` links (`q = Player()`), while this thread may test `w`; give `q` its new value after the `with scope` block
        if w exists as p:
            println(p.hp)
    println(a.len())
