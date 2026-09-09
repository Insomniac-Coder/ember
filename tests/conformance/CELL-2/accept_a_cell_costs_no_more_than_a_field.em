#$ test: run-pass
#$ rules: CELL-2, CELL-1
# "`get` is a load; `set` is a store. There is no overhead relative to a plain
# field." — and "no runtime check is needed".
#
# `Cell[T]` is a one-field struct and its operations lower to field accesses,
# so the claim is checked by looking for what must *not* be there: no runtime
# call carrying its name, and no check. The mutation through a shared borrow is
# real and is visible as a plain C assignment.

fn main():
    c: Cell[i32] = Cell(1)
    c.set(2)
    println(c.get())
#$ stdout: 2
#$ assert-c: contains(".value = ")
#$ assert-c: !contains("ember_cell")
#$ assert-c: !contains("cell_borrow")
