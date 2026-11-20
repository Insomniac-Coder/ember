# Clause BCK-1.1: "Each value that may hold views carries the set of views it may hold; copying,
# passing, returning, storing or projecting a value carries its views with it, and a call's result,
# and a view a call puts in through a `ref mut` parameter, carry the views of the arguments they may
# point into (`[LT-1]`, `from`)."
#$ test: compile-fail
#$ rules: BCK-1, BCK-4, LT-1, LT-47
# A copy, a projection, a call's result and a view put in through `ref mut` each still view `ys`
# after a push may move it. No text in the spec for these programs: code only.

struct Holder:
    view: Span[int]

fn longer(a: Span[int], b: Span[int]) -> Span[int]:
    if a.len() >= b.len():
        return a
    return b

fn put_longer(ref mut out: Span[int], other: Span[int]):
    if other.len() > out.len():
        out = other

fn copied():
    ys = [1, 2]
    h = Holder(view=ys)
    g = h
    ys.push(owns 3)
    println(g.view[0])    #$ error[E3060]

fn projected():
    ys = [1, 2]
    h = Holder(view=ys)
    v = h.view
    ys.push(owns 3)
    println(v[0])    #$ error[E3060]

fn by_result():
    xs = [1]
    ys = [1, 2]
    r = longer(xs, ys)
    ys.push(owns 3)
    println(r[0])    #$ error[E3060]

fn by_ref_mut():
    xs = [1]
    ys = [1, 2]
    out: Span[int] = xs
    put_longer(ref mut out, ys)
    ys.push(owns 3)
    println(out[0])    #$ error[E3060]

fn main():
    copied()
    projected()
    by_result()
    by_ref_mut()
