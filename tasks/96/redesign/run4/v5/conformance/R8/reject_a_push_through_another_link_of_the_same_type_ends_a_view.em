#$ test: compile-fail
#$ rules: BRW-17
#$ help: if a is not b:
## design rules: R8, R9, R5
## R8 (b): two paths may reach the same object when they lead to values of one type and at least one
## goes through a link, unless the compiler knows they differ; a push through `b` may move the list
## `s` points into through `a`.

class Bag:
    items: Array[int] = []

fn refill(owns a: shares mut Bag, owns b: shares mut Bag):
    s = a.items.as_span()
    b.items.push(4)
    println(s[0])    #$ error[E3060]: `b.items.push(4)` (line 13) may move the list `s` points into (`a.items`, line 12), since `a` and `b` may be the same `Bag`, and `s` is used on line 14; use `s` before line 13, or put lines 12 to 14 inside `if a is not b:`

fn main():
    b1 = shares mut Bag(items=[1, 2])
    b2 = shares mut Bag(items=[3])
    refill(shares mut b1, shares mut b2)
