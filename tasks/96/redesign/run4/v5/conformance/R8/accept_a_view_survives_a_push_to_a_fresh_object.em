#$ test: run-pass
#$ rules: R8, R9, R5
#$ stdout: 1 1
## R8 (b): an object made in this function after the view is a fresh object, which no other path can
## reach, so a push to it does not end a view into `a`.

class Bag:
    items: Array[int] = []

fn refill(owns a: shares mut Bag):
    s = a.items.as_span()
    fresh = shares mut Bag()
    fresh.items.push(4)
    println(s[0], fresh.items.len())

fn main():
    b = shares mut Bag(items=[1, 2])
    refill(shares mut b)
