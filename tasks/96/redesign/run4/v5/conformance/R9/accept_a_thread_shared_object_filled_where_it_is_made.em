#$ test: compile-pass
#$ rules: RC-1, THR-1
## design rules: R9, R15
## R9: the nearest allowed program: the list is filled on the line that makes the `thread_shared`
## object, and afterwards every owner only reads it (AN-71's message, applied).

import std.thread

class Names:
    items: Array[String] = []

fn share_and_keep(s: String):
    names = thread_shared Names(items=[copy of s])
    h = thread.spawn(fn() thread_shared names: names.items.len())
    println(names.items.len(), h.join())

fn main():
    share_and_keep(String("ann"))
