#$ test: compile-fail
#$ rules: R14
## R14: "a method its bounds' interfaces do not declare is E2072" (section 3's E2072 message, AN-69).

struct Crate:
    kg: int

    fn weight() -> int:
        return this.kg

[T]
fn heaviest(xs: Span[T]) -> int:
    best = 0
    for x in xs:
        w = x.weight()    #$ error[E2072]: `T` has no `weight`: no interface in scope declares `fn weight`; declare one (`interface Weighed: fn weight() -> int`) and the bound follows
        if w > best:
            best = w
    return best

fn main():
    crates = [Crate(kg=3), Crate(kg=5)]
    println(heaviest(crates.as_span()))
