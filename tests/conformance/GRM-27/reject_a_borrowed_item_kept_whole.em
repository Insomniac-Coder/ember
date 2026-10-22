#$ test: compile-fail
#$ rules: GRM-27, GRM-15
## `[GRM-27]` (ODR-236) — over a borrowed collection a comprehension's items are references: one
## kept whole would move out of the collection, which still owns it. The help names both ways:
## take the collection over (`for s in owned shapes`) or copy each item (`s.clone()`).

struct Shape:
    name: String
    size: int

fn main():
    shapes = [Shape("small", 1), Shape("big", 7)]
    big = [s for s in shapes if s.size > 2]  #$ error[E3013]: cannot move a borrowed item into the new collection
    #$ help: take the collection over, `for s in owned shapes`
    #$ help: or copy each item, `s.clone()`, `shapes` staying as it was
    println(len(big))
