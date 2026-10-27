#$ test: compile-fail
#$ rules: THR-17, THR-10
## design rules: R15
## R15: "An unscoped thread ... say[s] in the capture list what [it holds] (`owns`, `thread_shared`,
## `copy of`; E7004 for a name with no word)" (AN-71; section 3's E7004 message, with the lines to
## write). The nearest allowed programs are `accept_an_unscoped_thread_takes_its_object_over.em` and
## `accept_a_thread_shared_object_is_read_by_a_thread_and_its_maker.em`.

import std.thread

class Names:
    items: Array[String] = []

fn share_plain():
    names = Names()
    h = thread.spawn(fn() => names.items.len())    #$ error[E7004]: this thread may outlive `share_plain`, so it cannot use `share_plain`'s `names`; say what the thread holds: write `h = thread.spawn(fn() owns names => names.items.len())` in place of line 15 (handed over), or create it shared, `names = thread_shared Names()` in place of line 14, and write `h = thread.spawn(fn() thread_shared names => names.items.len())` in place of line 15 (one more owner)
    println(h.join())

fn main():
    share_plain()
