#$ test: compile-fail
#$ rules: THR-11, THR-9
#$ error[E7005]: the task borrows `c`, a `RefCell[i64]`, which is not Sync
# `[THR-11]` — what a task borrows shared, another thread reads with this one: it must be `Sync`.

import std.thread

fn main():
    c = RefCell(0)
    with scope = thread.scope():
        scope.spawn(fn() => c.borrow().clone())
