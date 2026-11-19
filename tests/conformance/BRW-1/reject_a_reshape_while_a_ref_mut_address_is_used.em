# Clause BRW-1.6: "A reshape through another name while a `ref mut` name that is not a position is
# still used is `E3022`, at the reshape; a position is ruled by `[BRW-16]`."
#$ test: compile-fail
#$ rules: BRW-1, BRW-17, BCK-3, SPN-3, LT-1
# Two `ref mut` addresses: a name holding a `MutSpan`, and a view a call gives. Codes only.

fn first(ref mut xs: Array[int]) -> ref mut int:
    return ref mut xs[0]

fn a_span():
    xs = [1, 2, 3]
    d = ref mut xs.as_mut_span()
    xs.push(owns 4)    #$ error[E3022]
    d[0] = 0

fn a_view_a_call_gives():
    xs = [1, 2, 3]
    r = ref mut first(ref mut xs)
    xs.clear()    #$ error[E3022]
    r = 5

fn main():
    a_span()
    a_view_a_call_gives()
