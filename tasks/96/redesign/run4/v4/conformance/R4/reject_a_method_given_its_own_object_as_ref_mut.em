#$ test: compile-fail
#$ rules: R4, R5
#$ help: a.merge(ref mut b)
## R4: the object a method is called on counts as one of the call's arguments, so `a.merge(ref mut a)`
## gives `merge` two names for one bag, both changeable: `merge` would push to the list it walks
## (completeness G18, E3022).

class Bag:
    items: Array[int] = []

    fn merge(ref mut other: Bag) changes items:
        for x in other.items:
            this.items.push(x)
        other.items.clear()

fn main():
    a = Bag(items=[1, 2])
    a.merge(ref mut a)    #$ error[E3022]: `merge` changes `a` (its first line says `changes items`) and `other`, and both are `a`; give it another bag: `b = Bag(items=[3])` above this line, and `a.merge(ref mut b)`
    println(a.items.len())
