#$ test: compile-fail
#$ rules: TYP-24, DIA-14
# D-432 — after `E2070` the call has no type to check against: it was
# checked against the first candidate, adding "expected `bool`, found
# `Array[i64]`" about a choice the compiler made itself, and a generic body
# reported its `E2070` twice (checked opaquely, then for the instance).

interface Conv[T]:
    fn conv(self) -> T

struct Gadget:
    n: int

extend Gadget implements Conv[bool]:
    fn conv(self) -> bool:
        return true

extend Gadget implements Conv[Array[int]]:
    fn conv(self) -> Array[int]:
        return [self.n]

fn pick[T](g: Gadget, x: T) -> bool:
    return g.conv()    #$ error[E2070]: `conv` is offered by both `Conv[bool]` and `Conv[Array[i64]]`

fn main():
    g = Gadget(n = 1)
    a: Array[int] = g.conv()    #$ error[E2070]: `conv` is offered by both `Conv[bool]` and `Conv[Array[i64]]`
    println(a.len(), pick(g, 1))
