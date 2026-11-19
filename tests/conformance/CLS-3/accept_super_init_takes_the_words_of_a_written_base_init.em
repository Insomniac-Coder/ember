#$ test: run-pass
#$ rules: CLS-3, FN-2a
#$ stdout: 1 2 1
# `[CLS-3]` — the arguments of `super.init(…)` take the words a call of that constructor takes:
# `ref mut` before one for its `ref mut` parameter.

open class Base:
    n: int

    fn init(ref mut count: int):
        count += 1
        this.n = count

class Derived(Base):
    m: int

    fn init(ref mut count: int):
        this.m = 2
        super.init(ref mut count)

fn main():
    count = 0
    d = Derived(ref mut count)
    println(d.n, d.m, count)
