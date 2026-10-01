#$ test: compile-fail
#$ rules: TYP-24, TYP-21
# D-402 — two instances of one generic interface are told apart by their
# arguments (D-313); a call whose arguments fit both chooses neither and is
# `E2070`. It took whichever was registered last, so `a: Array[int] =
# w.conv()` was a type mismatch against `Conv[bool]`'s `conv`.

interface Conv[T]:
    fn conv(self) -> T

struct W:
    x: int

extend W implements Conv[Array[int]]:
    fn conv(self) -> Array[int]:
        return [self.x]

extend W implements Conv[bool]:
    fn conv(self) -> bool:
        return true

fn main():
    w = W(x = 5)
    a: Array[int] = w.conv()    #$ error[E2070]: `conv` is offered by both `Conv[Array[i64]]` and `Conv[bool]`
    #$ help: name the one to call: `Conv[Array[i64]].conv(…)` or `Conv[bool].conv(…)`, with the receiver first
    println(a.len())
