#$ test: compile-fail
#$ rules: CELL-4, OWN-3
# "`Cell[T]` for a non-`Copy` `T` is move-only, and is `Drop` iff `T` is."
#
# `Bag` owns a heap buffer, so `Cell[Bag]` must move. The same derivation that
# made the `Copy` case work gives this one: `is_copy` asks about the field, the
# field is not `Copy`, so neither is the cell — and the read after the move is
# a use of a moved value.

struct Bag:
    pub v: Array[i32]

fn main():
    a: Cell[Bag] = Cell(Bag(Array[i32]()))
    b = a
    c = a.replace(Bag(Array[i32]()))   #$ error[E3040]: `a` has been moved out of
    println(0)
