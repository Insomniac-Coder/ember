#$ test: compile-fail
#$ rules: R15, R12, R11
#$ help: give the thread a copy of its own: write `h = thread.spawn(fn() copy of names: names.items.len())` in place of line 16
## R15, R12, R11 (step 4's safety re-check): a class object that a `weakref` link on this thread points
## at is not handed whole to another thread (E7015): this thread could open `w` while the other thread
## ends `names`, a race on the object's one-thread header and a read of freed contents.

import std.thread

class Names:
    items: Array[String] = []

fn main():
    names = Names()
    w = weakref names
    h = thread.spawn(fn() owns names: names.items.len())    #$ error[E7015]: `names` goes to another thread, and `w` (line 15), a `weakref` link on this thread, still points at it, so this thread could read it while the other thread ends it; give the thread a copy of its own: write `h = thread.spawn(fn() copy of names: names.items.len())` in place of line 16
    if w exists as p:
        println(p.items.len())
    println(h.join())
