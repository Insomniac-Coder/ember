# Clause THR-1.1: "`thread_shared x` makes one more owner of `x`, which may be on any thread; it is
# written before a new value (...), on a value already `thread_shared` (...) or before `this` in a
# method (...), and on a value made otherwise it is `E7017`"
#$ test: compile-fail
#$ rules: THR-1, RC-1
# The spec's message.

class Names:
    items: Array[String] = []

fn share_and_keep():
    names = Names()
    t = thread_shared names    #$ error[E7017]: `names` is `share_and_keep`'s own object, which it may change; to share it between threads, create it shared: `names = thread_shared Names()`
    println(t.items.len())

fn main():
    share_and_keep()
