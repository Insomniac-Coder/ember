#$ test: compile-fail
#$ rules: TYP-17, IFC-4
# D-416 — an extension applies only to an instance meeting its bounds, the
# associated-type bindings included: `Holder[Words]`, whose `Item` is `str`,
# took the methods and the interface of
# `extend[T: Iterator[Item = int]] Holder[T] implements Mark`.

interface Mark:
    fn mark(self) -> int

struct Words:
    n: int

extend Words implements Iterator:
    type Item = str
    fn next(mut self) -> Option[str]:
        return None

struct Holder[T]:
    x: T

extend[T: Iterator[Item = int]] Holder[T] implements Mark:
    fn mark(self) -> int:
        return 1

fn main():
    h = Holder(x = Words(n = 0))
    println(h.mark())        #$ error[E1010]: `Holder[Words]` has no method named `mark`
