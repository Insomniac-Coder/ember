#$ test: compile-fail
#$ rules: R15
## R15: "a thread-bound value sent away is E7008" (section 3's message, with every line it replaces
## named, R18). The scope is opened before the guard, so the way out does not end the scope inside the
## lock its task takes (E7018; check-V6's fix). The nearest allowed program is
## `accept_a_task_takes_the_lock_itself.em`.

import std.thread
from std.sync import Mutex

fn main():
    m = Mutex([0])
    with scope = thread.scope():
        with g = m.lock():
            g.push(2)
            scope.spawn(fn(): g.push(1))    #$ error[E7008]: `g` is the guard of `m`'s lock, which belongs to the thread that took it; take the lock inside the task: write `fn add_one(m: Mutex[Array[int]]):` with the body `with g = m.lock():` and, inside it, `g.push(1)`, above line 11, and `scope.spawn(fn(): add_one(m))` in place of line 16
    with g = m.lock():
        println(g.len())
