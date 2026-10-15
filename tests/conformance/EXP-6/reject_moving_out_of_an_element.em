#$ test: compile-fail
#$ rules: EXP-6, SOA-6
#$ error[E3011]: cannot move out of an element
#$ error[E3011]: cannot move out of an element
# `[EXP-6]` — an element of an `Array` (or an `SoA` column) stays in its container, which drops it
# with the rest: moving it out is `E3011` (D-599: it was accepted, and the rest of the array leaked).

struct Named:
    name: String
    score: int

fn main():
    xs = ["a".to_string(), "b".to_string()]
    taken = xs[0]
    ns = SoA[Named].with_capacity(1)
    ns.push(Named("ann".to_string(), 3))
    name = ns[0].name
    println(taken, name)
