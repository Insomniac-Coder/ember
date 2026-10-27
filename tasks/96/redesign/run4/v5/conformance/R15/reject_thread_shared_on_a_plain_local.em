#$ test: compile-fail
#$ rules: R15, R3
## R3: "`thread_shared x` ... only before a new value ... or on a value already `thread_shared`, else
## E7017" (AN-71).

import std.thread

class Names:
    items: Array[String] = []

fn share_plain():
    names = Names()
    h = thread.spawn(fn() thread_shared names: names.items.len())    #$ error[E7017]: `names` is `share_plain`'s own object, which it may change; to share it between threads, create it shared: write `names = thread_shared Names()` in place of line 12
    println(h.join())

fn main():
    share_plain()
