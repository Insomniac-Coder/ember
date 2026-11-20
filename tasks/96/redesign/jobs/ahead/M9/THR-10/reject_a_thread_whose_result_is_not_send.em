# Clause THR-10.1: "`thread.spawn` runs a function on a new thread (...) and `R` MUST be `Send`
# (`E7015`, `E7008`)"
#$ test: compile-fail
#$ rules: THR-10, THR-8, THR-2
# The thread's result is a `shares` owner, which counts on one thread. No text in the spec: code
# only. (The clause's `E3020` for a thread started in a walk of what it changes is Q2: no program
# found that reaches it.)

import std.thread

class Player:
    hp: int = 5

fn main():
    h = thread.spawn(owns fn(): shares Player())    #$ error[E7015]
    p = h.join()
    println(p.hp)
