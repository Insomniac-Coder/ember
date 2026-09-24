#$ test: compile-fail
#$ rules: LT-1, BRW-8
# A view parameter is passed as a copy (`[BRW-8]`): what its views point to is
# the caller's, but an `Array` or `Box` it owns is its own storage. An
# `owned` one frees it at return; a borrowed one shares it with no loan behind
# it. A reference into either is `E3060`; an element of its `Span` is fine.

struct P:
    x: int

@view
struct H:
    s: Span[int]
    arr: Array[int]
    b: Box[P]

fn from_span(h: H) -> ref int:
    return ref h.s[0]

fn from_array(owned h: H) -> ref int:
    return ref h.arr[0]    #$ error[E3060]: `h.arr[0]` does not live long enough

fn from_box(h: H) -> ref int:
    return ref h.b.x    #$ error[E3060]: `h.b.x` does not live long enough

fn main():
    println(0)
