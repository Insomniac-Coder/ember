#$ test: run-pass
#$ rules: SPN-5, LT-1, BRW-8
#$ stdout: 20
#$ 20
#$ 3
# `split_at` and `iter_mut` on a view parameter hand back views of what the
# view points to, the caller's memory, never of the parameter's own slot: a
# `mut` `MutSpan`, and a view struct taken `owned` whose `Array` field is
# dropped at return while its span's items live on.

from std.collections import MutSpanIter

@view
struct H:
    s: MutSpan[int]
    arr: Array[int]

fn tail(mut s: MutSpan[int]) -> MutSpan[int]:
    parts = s.split_at(1)
    return parts.1

fn items(owned h: H) -> MutSpanIter[int]:
    return h.s.iter_mut()

fn main():
    xs: Array[int] = Array[int]()
    xs.push(1)
    xs.push(2)
    t = tail(xs.as_mut_span())
    t[0] = 20
    println(t[0])
    println(xs[1])
    ys: Array[int] = Array[int]()
    ys.push(3)
    i = items(H(s=ys.as_mut_span(), arr=Array[int]()))
    match i.next():
        Some(r):
            println(r)
        None:
            println("none")
