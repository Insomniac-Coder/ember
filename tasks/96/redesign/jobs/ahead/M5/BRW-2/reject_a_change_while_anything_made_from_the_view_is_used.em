# Clause BRW-2.1: "A view is in use from where it is made to the last use of anything made from it:
# the view itself, a view made from it, a value returned from a call it was given to."
#$ test: compile-fail
#$ rules: BRW-2, BRW-1, BCK-2, LT-1
# `v` itself is not used after the change; what was made from it is. Codes only.

fn head(xs: Span[int]) -> Span[int]:
    return xs[0..1]

fn a_view_made_from_it():
    xs = [1, 2, 3]
    v = xs.as_span()
    w = v[1..]
    xs[1] = 9    #$ error[E3021]
    println(w[0])

fn a_value_a_call_returned():
    xs = [1, 2, 3]
    v = xs.as_span()
    h = head(v)
    xs[0] = 9    #$ error[E3021]
    println(h[0])

fn main():
    a_view_made_from_it()
    a_value_a_call_returned()
