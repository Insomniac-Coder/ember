#$ test: compile-fail
#$ rules: TYP-24, DIA-2
# D-415 — a diagnostic names an interface as a program writes it: a prelude
# interface without `std.core.`, an instance with its arguments
# (`Conv[str]`, not the internal `Conv_str`), and E2070's help lists every
# instance that fits as well.

interface Conv[T]:
    fn conv(self) -> T

struct W:
    x: int

extend W implements Conv[bool]:
    fn conv(self) -> bool:
        return true

extend W implements Conv[int]:
    fn conv(self) -> int:
        return 1

fn main():
    w = W(x = 1)
    a = Conv[str].conv(w)    #$ error[E2040]: `W` has no `conv` from `Conv[str]`
    b = w.conv()    #$ error[E2070]: `conv` is offered by both `Conv[bool]` and `Conv[i64]`
    #$ help: name the one to call: `Conv[bool].conv(…)` or `Conv[i64].conv(…)`, with the receiver first
    c = Conv.conv(w)    #$ error[E2070]: `conv` is offered by both `Conv[bool]` and `Conv[i64]`
