#$ test: compile-fail
#$ rules: R15
## R15: "a thread-bound value sent away is E7008" (section 3's message).

import std.thread
from std.sync import Mutex

fn main():
    m = Mutex([0])
    with g = m.lock():
        with scope = thread.scope():
            scope.spawn(fn() changes g: g.push(1))    #$ error[E7008]: `g` is the guard of `m`'s lock, which belongs to the thread that took it; take the lock inside the task: `with g = m.lock():`
