#$ test: run-pass
#$ rules: R15, R3, R9
#$ stdout: 1 1
## R15: `fn() thread_shared names` makes one more owner, on any thread, of a value created
## `thread_shared`; neither owner changes it directly (AN-71's `share_and_keep`).

import std.thread

class Names:
    items: Array[String] = []

fn share_and_keep(s: String):
    names = thread_shared Names(items=[copy of s])
    h = thread.spawn(fn() thread_shared names: names.items.len())
    println(names.items.len(), h.join())

fn main():
    share_and_keep(String("ann"))
