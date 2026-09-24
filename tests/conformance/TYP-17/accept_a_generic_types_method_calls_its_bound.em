#$ test: run-pass
#$ rules: TYP-17, TYP-24
#$ stdout: 102 103
# `[TYP-17]` — inside `Wrapper[T: Sized]`, `self.item.len()` is `Sized.len`,
# for every `T`, even one with its own `len` (D-248). A generic signature
# naming `Wrapper[T]` makes an instance over its own `T`, whose methods are
# not checked against that function's bounds (D-250: `through` was refused,
# "`T` has no method `len`").

interface Sized:
    fn len(self) -> int

struct Bag:
    n: int

    fn len(self) -> int:
        return self.n

extend Bag implements Sized:
    fn len(self) -> int:
        return self.n + 100

struct Wrapper[T: Sized]:
    item: T

    fn size(self) -> int:
        return self.item.len()

fn through[T: Sized](w: Wrapper[T]) -> int:
    return w.size()

println(Wrapper(Bag(2)).size(), through(Wrapper(Bag(3))))
