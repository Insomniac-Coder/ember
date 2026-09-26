#$ test: run-pass
#$ rules: MOD-5
#$ stdout: 3 4 5
#$ stdout: 7 1 2
# `[MOD-5]` — a local declaration shadows a prelude name, in type position as
# in construction. `Cell`, `Box` and `Option` below are the program's own;
# `Some(2)` is still the prelude's `Option` (D-305: `Cell[int]` as a type was
# the compiler's `Cell`, a declared `Box` shared the compiler's instances, and
# a declared `Option` broke `std`'s own).

class Cell[T]:
    value: T

    fn make(v: T) -> Cell[T]:
        return Cell(value=v)

    fn get(self) -> T:
        return self.value

struct Box[T]:
    item: T

enum Option[T]:
    Nothing
    Just(T)

fn keep(c: Cell[int]) -> int:
    return c.get()

fn main():
    c = Cell[int](value=3)
    d: Cell[int] = Cell[int].make(4)
    b: Box[int] = Box(5)
    println(keep(c), d.get(), b.item)
    nested = Box(Box(7))
    real: Option[int] = Option[int].Just(1)
    match real:
        Option.Just(v):
            println(nested.item.item, v, Some(2).unwrap())
        Option.Nothing:
            println("none")
