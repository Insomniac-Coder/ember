#$ test: run-pass
#$ rules: THR-2, THR-8
#$ stdout: 3
# `[THR-8]` — an object is `Send` by its fields: with one owner, it may be handed to a thread.

import std.thread

class Plain:
    n: int = 3

fn main():
    a = Plain()
    t = thread.spawn(owns owned fn() => a.n)
    println(t.join())
