# Clause BCK-2.1: "A view is **live** at a point when some value carrying it is used at a later point
# on some path from that point, with no intervening assignment of that value. A view carried by a
# function's result is live on the paths that reach the `return`, and only on those paths."
#$ test: run-pass
#$ rules: BCK-2, BRW-13, BRW-2
#$ stdout: a
#$ stdout: c
#$ stdout: b
#$ stdout: new
# `v` is given another view before `names` is cleared, so the old view is not live there; `pick`
# returns a view of `xs` on one path and pushes to `xs` on the other, where no view is carried yet.

fn pick(ref mut xs: Array[String], k: int) -> StrView:
    if k < xs.len():
        return xs[k]
    xs.push(owns String("new"))
    return xs[xs.len() - 1]

fn main():
    names = [String("a")]
    others = [String("c")]
    v = names[0]
    println(v)
    v = others[0]
    names.clear()
    println(v)
    words = [String("b")]
    println(pick(ref mut words, 0))
    println(pick(ref mut words, 5))
