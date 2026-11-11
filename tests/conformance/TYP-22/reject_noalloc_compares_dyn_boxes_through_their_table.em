#$ test: compile-fail
#$ rules: TYP-22, EFF-1, TYP-36
# `[EFF-1]`, IX.1 (ODR-241) — two `Box[dyn I]` compare through their table's `eq`, which runs the
# `==` of a type the comparison does not name, so its effects are every effect, as a call through a
# `dyn` value's are: a `@noalloc` function may not compare two (`E4001`). Two `Box[int]` compare
# their numbers and allocate nothing.

interface Shape: needs Eq:
    fn area() -> int

@derive(Eq)
struct Square:
    side: int

extend Square implements Shape:
    fn area() -> int:
        return this.side

@noalloc
fn same(a: owns dyn Shape, b: owns dyn Shape) -> bool:
    return a == b  #$ error[E4001]: `@noalloc` function `same` reaches an allocation

@noalloc
fn same_int(a: owns int, b: owns int) -> bool:
    return a == b

fn main():
    a: owns dyn Shape = owns Square(1)
    one: owns int = owns 1
    println(same(a, a), same_int(one, one))
