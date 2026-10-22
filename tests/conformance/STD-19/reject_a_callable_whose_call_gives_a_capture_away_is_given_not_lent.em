#$ test: compile-fail
#$ rules: STD-19, CLO-6, OWN-3
# `[STD-19]` (ODR-240), `[CLO-6]` — a named callable is lent to an `Option` method only when it
# may be called again: one whose call gives a capture away ends with its one call, so it is given
# to `map`, and using it again is `E3040`. A named callable lent to `map` is the caller's again.

struct Token:
    value: int

fn main():
    o: Option[int] = Some(1)
    t = Token(2)
    taker = owned fn(owned x: int) -> int:
        kept = t
        return x + kept.value
    first = o.map(taker)
    second = o.map(taker)  #$ error[E3040]: `taker` has been moved out of
    keep = fn(owned x: int) -> int => x + 1
    third = o.map(keep)
    fourth = o.map(keep)
    println(first, second, third, fourth)
