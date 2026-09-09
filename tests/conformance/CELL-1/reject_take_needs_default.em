#$ test: compile-fail
#$ rules: CELL-1
# The members of `[CELL-1]` the compiler cannot reach yet, each refused with
# the reason rather than left to read as a missing method.
#
#  * `take(self) -> T where T: Default` — there is no `Default` interface in
#    the compiler at all. `CELL-DEF-1` in docs/BACKLOG.md.
#  * `get` and `update` are `T: Copy` only, which is the rule itself, not a
#    gap: "`get(self) -> T` is provided only where `T: Copy`". A cell hands out
#    no reference, so a non-`Copy` value can only be replaced.
#  * `set` writes through `self`, a borrow, and a borrow needs a place. Writing
#    into a value that dies at the end of the statement cannot be observed.

struct Bag:
    pub v: Array[i32]

fn main():
    c: Cell[i32] = Cell(1)
    y = c.take()              #$ error[E2020]: `take` needs `T: Default`, and `Default` is not built yet
    b: Cell[Bag] = Cell(Bag(Array[i32]()))
    x = b.get()               #$ error[E2020]: `get` on `Cell[Bag]` needs `Bag: Copy`
    Cell(5).set(6)            #$ error[E2140]: `set` needs a cell to write into, not a temporary
