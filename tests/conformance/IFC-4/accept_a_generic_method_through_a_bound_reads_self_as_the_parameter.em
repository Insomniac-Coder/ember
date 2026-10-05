#$ test: run-pass
#$ rules: IFC-4, TYP-17, STD-19
#$ stdout: 2
#$ stdout: 7
# D-437 (1) — an interface's generic method called through a type
# parameter's bound reads `Self` as the parameter: `it.zip(other)` is a
# `Zip[I, J]` (was `Zip[Self, J]`, which has no `count`), and `M.make(3)` is an
# `M` (was E2020, "expected `M`, found `Self`").

fn pairs[I: Iterator, J: Iterator](owned it: I, owned other: J) -> int:
    # G8-4 — `count()` gives the iterator's count type, `I.Count` here.
    return it.zip(other).count().to_int()

interface Make:
    fn make[T: Copy](x: T) -> Self

struct Thing:
    n: int

extend Thing implements Make:
    fn make[T: Copy](x: T) -> Thing:
        return Thing(n = 7)

fn build[M: Make]() -> M:
    return M.make(3)


fn main():
    println(pairs((0..3).iter(), (10..12).iter()))
    t: Thing = build[Thing]()
    println(t.n)
