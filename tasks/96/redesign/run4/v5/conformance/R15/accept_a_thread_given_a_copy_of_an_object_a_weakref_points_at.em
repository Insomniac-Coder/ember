#$ test: run-pass
#$ rules: THR-1, THR-2, THR-10, CLO-16, FFI-22
#$ stdout: 0
#$ stdout: 0
## design rules: R15, R12, R11
## R15, R12, R11: the nearest allowed program to
## `reject_an_object_a_weakref_points_at_handed_to_another_thread`: the thread gets a copy of its own,
## and `names`, which `w` points at, stays on this thread.

import std.thread

class Names:
    items: Array[String] = []

fn main():
    names = Names()
    w = weakref names
    h = thread.spawn(fn() copy of names: names.items.len())
    if w exists as p:
        println(p.items.len())
    println(h.join())
