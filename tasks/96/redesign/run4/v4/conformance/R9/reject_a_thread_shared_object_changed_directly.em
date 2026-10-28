#$ test: compile-fail
#$ rules: R9, R15
## R9: "A `thread_shared` value is never changed directly by any owner: only through `Atomic`, `Mutex`,
## `RwLock`, a channel, or a `guarded by` field under its lock" (AN-71's refused `share_and_keep`).

import std.thread

class Names:
    items: Array[String] = []

fn share_and_keep(s: String):
    names = thread_shared Names()
    names.items.push(copy of s)          #$ error[E7014]: `names` is shared between threads (line 12), so no owner changes it directly, and line 13 pushes to its list; fill the list where the object is made: write `names = thread_shared Names(items=[copy of s])` instead of lines 12 and 13
    h = thread.spawn(fn() with thread_shared names: names.items.len())
    println(names.items.len(), h.join())

fn main():
    share_and_keep(String("ann"))
