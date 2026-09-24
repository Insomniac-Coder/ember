#$ test: run-pass
#$ rules: TYP-17, TYP-24, GRM-34
#$ stdout: 2 102 2 102
# `[TYP-17]` — inside `size[S: Sized]`, `s.len()` is `Sized.len`: the bound
# is all a generic body may use. An instantiation checks the body again on
# the concrete type, and the call must still reach the interface's method,
# not the type's own `len` of the same name (`[TYP-24]` prefers that one
# only where the type is written). D-245.

interface Sized:
    fn len(self) -> int

struct Bag:
    n: int

    fn len(self) -> int:
        return self.n

extend Bag implements Sized:
    fn len(self) -> int:
        return self.n + 100

extend[T] Array[T] implements Sized:
    fn len(self) -> int:
        return self.len() + 100

fn size[S: Sized](s: S) -> int:
    return s.len()

xs = [1, 2]
println(Bag(2).len(), size(Bag(2)), xs.len(), size(xs))
