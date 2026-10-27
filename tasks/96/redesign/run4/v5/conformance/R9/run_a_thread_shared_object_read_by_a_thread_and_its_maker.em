#$ test: run-pass
#$ rules: R9, R15
#$ stdout: 1 1
## R9: "`thread_shared` counts are atomic": the thread's owner is one atomic +1 when the thread's
## function is made and one atomic -1 when it ends; both owners read the list (AN-71's
## `share_and_keep`; section 2's count row). Checked with ThreadSanitizer: no report.

import std.thread

class Names:
    items: Array[String] = []

fn main():
    names = thread_shared Names(items=[String("ann")])
    h = thread.spawn(fn() thread_shared names: names.items.len())
    println(names.items.len(), h.join())
