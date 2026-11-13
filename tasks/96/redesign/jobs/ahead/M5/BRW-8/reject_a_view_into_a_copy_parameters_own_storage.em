# Clause BRW-8.5: "A view into the own storage of a parameter passed as a copy ends with the call:
# returning it is `E3063`."
#$ test: compile-fail
#$ rules: BRW-8, LT-1, BCK-4
# The spec gives no message text for this case: the expectations name the code only. `w` (a view)
# and `n` (a small `Copy` value) are both passed as a copy. Q7 (INDEX.md): "own storage" is the
# copy's own bytes (`ref h.s`), not the blocks the caller's `H` owns, which outlive the call
# (`[LT-1]` names the two apart: "into its own storage or into storage it owns"), so
# `into_what_it_owns` is accepted; an `owns h` frees what it owns at the return.

struct P:
    x: int

@view
struct H:
    s: Span[int]
    arr: Array[int]
    b: owns P

fn keep(w: StrView) -> ref StrView:
    return ref w    #$ error[E3063]

fn keep_n(n: int) -> ref int:
    return ref n    #$ error[E3063]

fn into_its_own_bytes(h: H) -> ref Span[int]:
    return ref h.s    #$ error[E3063]

fn into_what_it_owns(h: H) -> ref int:
    return ref h.b.x

fn into_what_it_was_handed(owns h: H) -> ref int:
    return ref h.arr[0]    #$ error[E3063]

fn main():
    println(keep("a"), keep_n(1))
    xs = [1, 2]
    h = H(s=xs.as_span(), arr=[3], b=owns P(x=4))
    println(into_its_own_bytes(h).len(), into_what_it_owns(h))
    println(into_what_it_was_handed(owns H(s=xs.as_span(), arr=[5], b=owns P(x=6))))
