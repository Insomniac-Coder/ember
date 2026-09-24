#$ test: compile-fail
#$ rules: TYP-17
# `[TYP-17]` — "there is no duck typing": a method of `Wrapper[T: Sized]` may
# use only what `Sized` provides, even when every `T` the program uses has
# more. D-248: the method was checked only for each concrete `T`, so this
# compiled.

interface Sized:
    fn len(self) -> int

struct Bag:
    n: int

    fn frob(self) -> int:
        return self.n * 7

extend Bag implements Sized:
    fn len(self) -> int:
        return self.n

struct Wrapper[T: Sized]:
    item: T

    fn size(self) -> int:
        return self.item.frob()   #$ error[E2040]: `T` has no method `frob`; its bounds do not provide one

println(Wrapper(Bag(2)).size())
