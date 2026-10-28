#$ test: run-pass
#$ rules: BRW-16, RC-1, FN-4
#$ stdout: 1 2
## design rules: R8, R9, R5
## R8 (b): two paths that lead to values of different types never reach one object, so a push through
## the `Crate` does not end a view into the `Bag`.

class Bag:
    items: Array[int] = []

class Crate:
    items: Array[int] = []

fn refill(owns a: shares mut Bag, owns c: shares mut Crate):
    s = a.items.as_span()
    c.items.push(4)
    println(s[0], c.items.len())

fn main():
    b = shares mut Bag(items=[1, 2])
    c = shares mut Crate(items=[3])
    refill(shares mut b, shares mut c)
