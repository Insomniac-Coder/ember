#$ test: compile-fail
#$ rules: CLO-7, THR-10
#$ error[E3026]: the thread's function borrows `total`, which it may outlive
#$ help: borrow it inside `with scope = thread.scope():`, which joins its threads before the block ends [THR-5]
# `[CLO-7]` — a callback sent to another thread carries only the static region: a lambda that
# borrows a local is refused, and the help names the scope that may borrow.

import std.thread

fn main():
    total = 5
    worker = thread.spawn(fn() => total + 1)
