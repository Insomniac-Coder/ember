#$ test: compile-fail
#$ rules: BRW-7, OWN-3, DIA-14, TYP-5
# `[BRW-7]` — borrowing a local never given a value is `E3050`; borrowing one a move reached is
# `[OWN-3]`'s `E3040` (ODR-114); one error each, never both for one local.

fn peek(r: ref int) -> int:
    return r

fn size(r: ref String) -> int:
    return 0

fn main():
    x: int
    println(peek(x))    #$ error[E3050]: `x` is used before it is given a value
    s: String = "moved"
    t = s
    println(size(s))    #$ error[E3040]: `s` is borrowed after it has been moved out of
    println(t)
