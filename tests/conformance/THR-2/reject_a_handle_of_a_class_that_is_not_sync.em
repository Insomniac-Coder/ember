#$ test: compile-fail
#$ rules: THR-2, THR-10
#$ error[E7004]: the thread's function captures `a`, a `Plain`, which is not Send
#$ help: declare it `@sync class Plain`: its fields are all `Send` and `Sync` [THR-1]
#$ error[E7004]: the thread's function captures `b`, a `Holder`, which is not Send
#$ help: `@sync` would not help: `Holder.cell` is a `Cell[i64]`, which is not `Send` and `Sync` [THR-1]
# `[THR-2]` — a handle of a class that is not `@sync` cannot reach another thread; the diagnostic
# offers `@sync` when the class could be one, and names the field that keeps it from being one.

import std.thread

class Plain:
    n: int = 1

class Holder:
    cell: Cell[int] = Cell(0)

fn main():
    a = Plain()
    b = Holder()
    t1 = thread.spawn(owned fn() => a.n)
    t2 = thread.spawn(owned fn() => b.cell.get())
