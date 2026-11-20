# Clause TYP-17.1: "with none, its bounds are worked out from its body, each method called on a
# parameter giving the interface in scope that declares it (two that declare it are `E2070`,
# `[TYP-24]`)"
#$ test: compile-fail
#$ rules: TYP-17, IFC-6, TYP-24
# `Weighed` and `Heavy` both declare `fn weight`. No text in the spec: code only.

interface Weighed:
    fn weight() -> int

interface Heavy:
    fn weight() -> int

struct Crate implements Weighed, Heavy:
    w: int

    fn weight() -> int:
        return this.w

[T]
fn heaviest(xs: Span[T]) -> int:
    best = 0
    for x in xs:
        if x.weight() > best:    #$ error[E2070]
            best = x.weight()
    return best

fn main():
    println(heaviest([Crate(3)]))
