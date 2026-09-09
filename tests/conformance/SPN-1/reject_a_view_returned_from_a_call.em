#$ test: compile-fail
#$ rules: SPN-1, BRW-1, LT-1
# The loan has to survive a function boundary. `make_span` returns a view, so
# `[LT-1]`'s elision ties the result's region to the argument it came from —
# and `a` stays borrowed for as long as the returned view lives. Without that,
# a one-line wrapper would launder the borrow away and the push would compile.

fn make_span(xs: Span[i32]) -> Span[i32]:
    return xs

fn main():
    a: Array[i32] = Array[i32]()
    a.push(1)
    v = make_span(a)
    a.push(2)              #$ error[E3021]: `a` is borrowed here and mutably borrowed elsewhere
    println(v[0])
