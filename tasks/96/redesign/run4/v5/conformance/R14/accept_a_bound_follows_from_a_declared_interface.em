#$ test: run-pass
#$ rules: TYP-44
#$ stdout: 5
## design rules: R14
## R14: with the interface E2072's message gives declared, a non-`pub` body's bound is worked out from
## it.

interface Weighed:
    fn weight() -> int

struct Crate implements Weighed:
    kg: int

    fn weight() -> int:
        return this.kg

[T]
fn heaviest(xs: Span[T]) -> int:
    best = 0
    for x in xs:
        w = x.weight()
        if w > best:
            best = w
    return best

fn main():
    crates = [Crate(kg=3), Crate(kg=5)]
    println(heaviest(crates.as_span()))
