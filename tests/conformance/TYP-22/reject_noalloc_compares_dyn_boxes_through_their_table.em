#$ test: compile-fail
#$ rules: TYP-22, EFF-1, TYP-36
# `[EFF-1]`, IX.1 (ODR-241) — two `Box[dyn I]` compare through their table's `eq`, which runs the
# `==` of a type the comparison does not name, so its effects are every effect, as a call through a
# `dyn` value's are: a `@noalloc` function may not compare two (`E4001`). Two `Box[int]` compare
# their numbers and allocate nothing.

interface Shape: Eq:
    fn area(self) -> int

@derive(Eq)
struct Square:
    side: int

extend Square implements Shape:
    fn area(self) -> int:
        return self.side

@noalloc
fn same(a: Box[dyn Shape], b: Box[dyn Shape]) -> bool:
    return a == b  #$ error[E4001]: `@noalloc` function `same` reaches an allocation

@noalloc
fn same_int(a: Box[int], b: Box[int]) -> bool:
    return a == b

fn main():
    a: Box[dyn Shape] = Box(Square(1))
    println(same(a, a), same_int(Box(1), Box(1)))
