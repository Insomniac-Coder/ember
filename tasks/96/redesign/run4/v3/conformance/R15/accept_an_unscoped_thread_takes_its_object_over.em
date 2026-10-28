#$ test: run-pass
#$ rules: R15, R1
#$ stdout: 1
## R15: `with owns names` hands the object to the thread (AN-71's `share_plain`).

import std.thread

class Names:
    items: Array[String] = []

fn share_plain():
    names = Names(items=[String("ann")])
    h = thread.spawn(fn() with owns names: names.items.len())
    println(h.join())

fn main():
    share_plain()
