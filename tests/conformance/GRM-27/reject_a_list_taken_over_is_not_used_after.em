#$ test: compile-fail
#$ rules: GRM-27, GRM-15
## `[GRM-27]`, `[GRM-15]` (ODR-236) — a comprehension clause over `owned xs` takes the collection
## over, as a `for` does: `xs` cannot be used after.

struct Shape:
    name: String
    size: int

fn main():
    others = [Shape("a", 1)]
    taken = [s for s in owned others if s.size > 0]
    println(len(others), len(taken))  #$ error[E3040]: `others` has been moved out of
