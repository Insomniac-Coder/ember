#$ test: compile-fail
#$ rules: BRW-7, DIA-14, TYP-5
# `[BRW-7]` — no borrow of a moved or uninitialised place, one error each: a
# local never given a value is reported as that, not also as "moved out of".

fn peek(r: ref int) -> int:
    return r

fn size(r: ref String) -> int:
    return 0

fn main():
    x: int
    println(peek(x))    #$ error[E3050]: `x` is used before it is given a value
    s: String = "moved"
    t = s
    println(size(s))    #$ error[E3050]: `s` is borrowed after it has been moved out of
    println(t)
