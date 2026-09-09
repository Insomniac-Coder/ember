#$ test: run-pass
#$ rules: CELL-1
# "Its unconditional API is `Cell(owned v)`, `set(self, owned v: T)`,
# `replace(self, owned v: T) -> T`, `into_inner(owned self) -> T` … `get(self)
# -> T` is provided only where `T: Copy` … `update(self, f: fn(T) -> T)`."
#
# `take` needs `T: Default`, which the compiler cannot express yet — the gap is
# `CELL-DEF-1` in docs/BACKLOG.md, and `reject_take_needs_default.em` beside
# this one pins the message rather than leaving it as a missing method.
#
# Every one of these takes `self`, a *shared* borrow, and three of them mutate.
# That is the whole point of the type, and ADR-019 records what is permitted to
# do it.

fn bump(n: i32) -> i32:
    return n + 10

fn main():
    c: Cell[i32] = Cell(1)
    println(c.get())
    c.set(2)
    println(c.get())
    println(c.replace(3))
    println(c.get())
    c.update(bump)
    println(c.get())
    println(c.into_inner())
#$ stdout: 1
#$ 2
#$ 2
#$ 3
#$ 13
#$ 13
