#$ test: compile-fail
#$ rules: R8, R9, R5
#$ help: if a is not b:
## R8 (b): two paths may reach the same object when they lead to values of one type and at least one
## goes through a link, unless the compiler knows they differ; a push through `b` may move the list
## `s` points into through `a`.
## `refill` changes a `Bag` through a link it holds, so it says `changes Bag`, and so does `main`,
## which calls it (R5, verdicts 63, 64).

class Bag:
    items: Array[int] = []

fn refill(owns a: shares mut Bag, owns b: shares mut Bag) changes Bag:
    s = a.items.as_span()
    b.items.push(4)
    println(s[0])    #$ error[E3060]: `b.items.push(4)` (line 15) may move the list `s` points into (`a.items`, line 14), since `a` and `b` may be the same `Bag`, and `s` is used on line 16; use `s` before line 15, or put lines 14 to 16 inside `if a is not b:`

fn main() changes Bag:
    b1 = shares mut Bag(items=[1, 2])
    b2 = shares mut Bag(items=[3])
    refill(shares mut b1, shares mut b2)
