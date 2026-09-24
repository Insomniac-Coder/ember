#$ test: run-pass
#$ rules: TYP-24
#$ stdout: 2 102 102
# `[TYP-24]` — a type may have an inherent method and an interface method of
# one name: a call finds the inherent one; `I.m(x)` and a `dyn I` find the
# interface's. D-244: the interface's was dropped when it came second, and
# its body was compiled under the inherent one's C symbol.

interface Sized:
    fn len(self) -> int

struct Bag:
    n: int

    fn len(self) -> int:
        return self.n

extend Bag implements Sized:
    fn len(self) -> int:
        return self.n + 100

fn tell(s: ref dyn Sized) -> int:
    return s.len()

b = Bag(2)
println(b.len(), Sized.len(b), tell(ref b))
