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
    taker = owned fn(owns x: int) -> int:
        kept = t
        return x + kept.value
    first = o.map(owns taker)
    second = o.map(owns taker)  #$ error[E3040]: `taker` was handed over to `option_map` on line 16 and is used on line 17; to keep both, hand over a copy of your own: `copy of taker`
    keep = fn(owns x: int) -> int => x + 1
    third = o.map(owns keep)
    fourth = o.map(owns keep)
    println(first, second, third, fourth)
