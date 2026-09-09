#$ test: run-pass
#$ rules: CELL-4, OWN-2
# The other half of the same sentence: "and is `Drop` iff `T` is."
#
# `Cell[Bag]` has no destructor of its own and never gets one — what it has is
# a field that needs dropping, and `needs_drop` on a struct already asks that.
# So the cell's contents drop when the cell's scope ends, exactly once, and a
# `Cell[i32]` in the same function drops nothing at all.

struct Bag:
    pub v: Array[i32]

    fn drop(mut self):
        println(self.v.len())

fn make(n: i32) -> Bag:
    xs: Array[i32] = Array[i32]()
    i = 0
    while i < n:
        xs.push(i)
        i = i + 1
    return Bag(xs)

fn main():
    plain: Cell[i32] = Cell(5)
    owning: Cell[Bag] = Cell(make(2))
    println(plain.get())
#$ stdout: 5
#$ 2
