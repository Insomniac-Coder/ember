#$ test: run-pass
#$ rules: GRM-27, GRM-15, CTL-1
#$ profiles: debug, development, release
#$ stdout: dropped small
#$ kept 2: mid big
#$ copied 1, still 2
#$ dropped mid
#$ dropped big
#$ dropped small
#$ dropped mid
#$ dropped big
#$ {'big'} 7
## `[GRM-27]`, `[GRM-15]` (ODR-236) — a comprehension's `for` clause over `owned xs` takes the
## collection over, as a `for` does: the items it keeps move into the new collection, the rest are
## dropped as the clause passes them, and `xs` cannot be used after. Over a borrowed collection
## the items are references, which `.clone()` copies, the collection staying as it was. A set
## comprehension and a generator expression take theirs over the same way: one that keeps a
## clone of a part keeps no item, so each is dropped.

struct Shape:
    name: String
    size: int

    fn drop(mut self):
        println(f"dropped {self.name}")

fn shapes() -> Array[Shape]:
    return [Shape("small", 1), Shape("mid", 4), Shape("big", 7)]

fn main():
    all = shapes()
    kept = [s for s in owned all if s.size > 2]
    println(f"kept {len(kept)}: {kept[0].name} {kept[1].name}")
    names = [s.name.clone() for s in kept if s.size > 5]
    println(f"copied {len(names)}, still {len(kept)}")
    biggest = {s.name.clone() for s in owned kept if s.size > 5}
    total = sum(s.size for s in owned shapes() if s.size > 5)
    println(biggest, total)
