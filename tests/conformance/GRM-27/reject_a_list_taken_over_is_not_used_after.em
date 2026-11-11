#$ test: compile-fail
#$ rules: GRM-27, GRM-15
## `[GRM-27]`, `[GRM-15]` (ODR-236) — a comprehension clause over `owned xs` takes the collection
## over, as a `for` does: `xs` cannot be used after.

struct Shape:
    name: String
    size: int

fn main():
    others = [Shape("a", 1)]
    taken = [s for s in owns others if s.size > 0]
    println(len(others), len(taken))  #$ error[E3040]: `others` was handed over to `into_iter` on line 12 and is used on line 13; to keep both, hand over a copy of your own: `copy of others`
