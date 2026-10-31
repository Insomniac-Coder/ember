#$ test: compile-fail
#$ rules: IFC-4, TYP-17
# `[IFC-4]` (ODR-049, SP-003) — a default is not an equality: an
# implementation may override `Out`, so neither a bound `T: Combine` nor the
# interface's own default body may take `Out` to be `Self`. The type is
# named after its receiver, `T.Out` (D-345: it read `Self.Out`).

interface Combine:
    type Out = Self
    fn combine(other: Self) -> Out

    fn twice() -> Self:
        return this.combine(this) #$ error[E2020]: expected `Self`, found `Self.Out`

[T: needs Combine]
fn assume(a: T, b: T) -> T:
    return a.combine(b) #$ error[E2020]: expected `T`, found `T.Out`

fn main():
    println(1)
