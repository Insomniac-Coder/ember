#$ test: compile-fail
#$ rules: LT-1, BRW-8
#$ help: return `StrView` instead of `ref StrView`
# D-217 — a view parameter is passed as itself (`[BRW-8]`): the result may
# borrow what it points to, but a reference to the view is a reference into
# this frame's copy of it.

fn keep(x: StrView) -> ref StrView:
    return ref x    #$ error[E3060]: `x` does not live long enough

fn main():
    r = keep("zz")
    println(r)
