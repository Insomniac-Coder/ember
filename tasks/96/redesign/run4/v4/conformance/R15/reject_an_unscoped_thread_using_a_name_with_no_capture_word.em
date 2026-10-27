#$ test: compile-fail
#$ rules: R15
## R15: "An unscoped thread ... say[s] in the capture list what [it holds] (`with owns`,
## `with thread_shared`, `with copy of`; E7004 for a name with no word)" (AN-71).

import std.thread

class Names:
    items: Array[String] = []

fn share_plain():
    names = Names()
    h = thread.spawn(fn() => names.items.len())    #$ error[E7004]: this thread may outlive `share_plain`, so it cannot use `share_plain`'s `names`; say what the thread holds: `fn() with owns names:` (handed over), or create it shared, `names = thread_shared Names()`, and write `fn() with thread_shared names:` (one more owner)
    println(h.join())

fn main():
    share_plain()
