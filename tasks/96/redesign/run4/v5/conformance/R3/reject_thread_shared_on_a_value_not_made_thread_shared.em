#$ test: compile-fail
#$ rules: THR-1
#$ help: to share it between threads, create it shared: `names = thread_shared Names()`
## design rules: R3, R15
## R3, R15: `thread_shared x` is written only before a new value or on a value already
## `thread_shared` (E7017): a plain local's object has a one-thread count (AN-71's message).

class Names:
    items: Array[String] = []

fn share_plain():
    names = Names()
    keep = thread_shared names     #$ error[E7017]: `names` is `share_plain`'s own object, which it may change
    println(keep.items.len())

fn main():
    share_plain()
