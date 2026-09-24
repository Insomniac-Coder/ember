#$ test: compile-fail
#$ rules: BRW-3, BCK-5, TYP-5
# `[BRW-3]`'s other half: a borrow of the place that is itself an argument is
# live when the `mut` borrow activates. Both a `ref` parameter (auto-borrowed,
# `[TYP-5]` rule 7) and a borrowed `String` passed by address (`[BRW-8]`) are
# such a borrow; `refill` would clear the element `r` points to.

fn f(mut v: Array[int], x: ref int):
    v.push(x)

fn refill(mut dst: Array[String], r: String):
    dst.clear()
    println(r)

fn main():
    v: Array[int] = [5]
    f(v, v[0])    #$ error[E3021]: `v[0]` is borrowed here and mutably borrowed elsewhere
    xs: Array[String] = ["a"]
    refill(xs, xs[0])    #$ error[E3021]: `xs[0]` is borrowed here and mutably borrowed elsewhere
