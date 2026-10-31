#$ test: run-pass
#$ rules: TYP-24, TYP-17
#$ stdout: 7 7 21
# D-431 — in a generic body a receiver's bounds are told apart as a concrete
# receiver's implementations are: `Conv[bool].conv(c)` names an instance,
# `c.scale(2)` chooses `Scale[int]` by its argument (ODR-096), and `B.m(c)`
# names an interface. Each was `E2070`.

interface Conv[T]:
    fn conv() -> T

interface Scale[T]:
    fn scale(by: T) -> int

interface A:
    fn m() -> int

interface B:
    fn m() -> int

struct Gadget:
    n: int

extend Gadget implements Conv[bool]:
    fn conv() -> bool:
        return true

extend Gadget implements Conv[int]:
    fn conv() -> int:
        return 7

extend Gadget implements Scale[int]:
    fn scale(by: int) -> int:
        return this.n * by

extend Gadget implements Scale[bool]:
    fn scale(by: bool) -> int:
        if by:
            return 1
        return 0

extend Gadget implements A:
    fn m() -> int:
        return 1

extend Gadget implements B:
    fn m() -> int:
        return 2

[C: needs (Conv[bool], Conv[int])]
fn named(c: C) -> int:
    flag = Conv[bool].conv(c)
    n = Conv[int].conv(c)
    if flag:
        return n
    return 0

[C: needs (Scale[int], Scale[bool])]
fn chosen(c: C) -> int:
    return c.scale(2) + c.scale(true)

[C: needs (A, B)]
fn which(c: C) -> int:
    return B.m(c) * 10 + A.m(c)

fn main():
    g = Gadget(n = 3)
    println(named(g), chosen(g), which(g))
