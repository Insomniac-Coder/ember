# Clause THR-17.1: "An unscoped thread (`thread.spawn`), a job submitted outside a scope (`[JOB-2]`)
# and a callback C keeps (Part XVI) say in their capture list everything of their maker's that they
# use: `owns x` (...), `thread_shared x` (...), or `copy of x` (...); a name used with no word is
# `E7004`"
#$ test: compile-fail
#$ rules: THR-17, THR-10, CLO-2
# The spec's message.

import std.thread

class Names:
    items: Array[String] = []

fn share_plain():
    names = Names()
    h = thread.spawn(owns fn(): names.items.len())    #$ error[E7004]: this thread may outlive `share_plain`, so it cannot use `share_plain`'s `names`; say what the thread holds: `fn() owns names:` (handed over), or create it shared, `names = thread_shared Names()`, and write `fn() thread_shared names:` (one more owner)
    println(h.join())

fn main():
    share_plain()
