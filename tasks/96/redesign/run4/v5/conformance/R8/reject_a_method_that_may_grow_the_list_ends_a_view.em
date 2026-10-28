#$ test: compile-fail
#$ rules: BRW-17
#$ help: if a is not b:
## design rules: R8, R5, R9
## R8 (b): what may reshape is worked out from the callee's body (R5, verdict 86): `add` pushes to
## `this.items`, and `b` may be the same `Bag` as `a`, so the call may move the list `s` points into.

class Bag:
    items: Array[int] = []

    fn add(x: int):
        this.items.push(x)

fn refill(owns a: shares mut Bag, owns b: shares mut Bag):
    s = a.items.as_span()
    b.add(4)
    println(s[0])    #$ error[E3060]: `b.add(4)` (line 15) may move the list `s` points into (`a.items`, line 14): `add` pushes to `this.items` (line 11), and `a` and `b` may be the same `Bag`; `s` is used on line 16; use `s` before line 15, or put lines 14 to 16 inside `if a is not b:`

fn main():
    b1 = shares mut Bag(items=[1, 2])
    b2 = shares mut Bag(items=[3])
    refill(shares mut b1, shares mut b2)
