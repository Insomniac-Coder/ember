#$ test: run-pass
#$ rules: CELL-4
# "`Cell[T]` is `Copy` when `T: Copy`, and copying such a `Cell` copies the
# value it holds at that moment."
#
# Both halves are asserted: that `b = a` is a copy rather than a move — if it
# moved, reading `a` afterwards would be `E3040` — and that the copy is of the
# value, not of the cell, so writing through `a` later does not reach `b`.
#
# No code implements this. `Cell` is a struct with `derives_copy` set and no
# `drop`, and `is_copy` on a struct already asks whether every field is `Copy`,
# so the question reduces to the same question about `T`.

fn main():
    a: Cell[i32] = Cell(1)
    b = a
    a.set(7)
    println(a.get())
    println(b.get())
#$ stdout: 7
#$ 1
