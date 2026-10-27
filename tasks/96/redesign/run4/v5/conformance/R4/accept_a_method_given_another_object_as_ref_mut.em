#$ test: run-pass
#$ rules: FN-1, FN-4
#$ stdout: 3 0
## design rules: R4, R5
## R4: the nearest allowed program: `a` and `b` are two bags, so the method's object and its `ref mut`
## argument name two places (completeness G18).

class Bag:
    items: Array[int] = []

    fn merge(ref mut other: Bag):
        for x in other.items:
            this.items.push(x)
        other.items.clear()

fn main():
    a = Bag(items=[1, 2])
    b = Bag(items=[3])
    a.merge(ref mut b)
    println(a.items.len(), b.items.len())
