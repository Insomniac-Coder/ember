#$ test: run-pass
#$ rules: LT-1, LT-22, DSP-5
#$ stdin: b
#$ stdout: [7] [1, 2]
#$ stdout: [8] [1, 2]
#$ stdout: [8] [1, 2]
# `[LT-1]` (verdict 151) — a `virtual` method's call is decided by the code of each override that may
# run: `A.first` and `B.first` both give back the object's own list, so `r` points into `a` only and
# `xs` may change while `r` is used, for an `A`, for a `B`, and for an `A` whose class is chosen
# while running.

open class A:
    let items: Array[int] = [7]

    virtual fn first(other: Array[int]) -> Span[int]:
        return this.items

class B(A):
    let more: Array[int] = [8]

    override fn first(other: Array[int]) -> Span[int]:
        return this.more

fn show(a: A):
    xs = [1]
    r = a.first(xs)
    xs.push(owns 2)
    println(r, xs)

fn main():
    show(A())
    show(B())
    a: A = B() if input() == "b" else A()
    show(a)
