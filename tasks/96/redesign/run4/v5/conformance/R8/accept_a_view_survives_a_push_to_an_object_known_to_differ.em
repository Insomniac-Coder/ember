#$ test: run-pass
#$ rules: R8, R9, R5
#$ stdout: 1
## R8 (b): inside `if a is not b:` the compiler knows the two links name different objects, so a push
## through `b` does not end a view into `a`.

class Bag:
    items: Array[int] = []

fn refill(owns a: shares mut Bag, owns b: shares mut Bag):
    if a is not b:
        s = a.items.as_span()
        b.items.push(4)
        println(s[0])

fn main():
    b1 = shares mut Bag(items=[1, 2])
    b2 = shares mut Bag(items=[3])
    refill(shares mut b1, shares mut b2)
