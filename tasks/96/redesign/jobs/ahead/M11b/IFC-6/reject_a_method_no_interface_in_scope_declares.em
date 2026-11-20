# Clause IFC-6.1: "a method that no interface in scope declares is `E2072`"
#$ test: compile-fail
#$ rules: IFC-6, TYP-17
# The spec's message up to where IFC-6's and TYP-17's texts part ("for the type `heaviest` is
# given" against "for the type the function is given", Q3).

struct Crate:
    w: int

    fn weight() -> int:
        return this.w

[T]
fn heaviest(xs: Span[T]) -> int:
    best = 0
    for x in xs:
        if x.weight() > best:    #$ error[E2072]: `T` has no `weight`: no interface in scope declares `fn weight`; declare one, `interface Weighed:` with `fn weight() -> int`, and write `struct Crate implements Weighed:` for the type
            best = x.weight()
    return best

fn main():
    println(heaviest([Crate(3)]))
