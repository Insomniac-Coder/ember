#$ test: run-pass
#$ rules: IFC-4, TYP-17
#$ stdout: 6
# D-437 (2) — an instance of a generic interface over the caller's own
# parameter (`Gather[T]`) numbers its method's own parameters after the
# caller's, so `C.gather(it)` no longer reads the method's `I` in the
# caller's slot (was E2040, "`I`'s `Item` for `Iterator` is `T`, but `I`'s bound
# needs `I`").

interface Gather[T]:
    [I: needs Iterator[Item = T]]
    static fn gather(owns it: I) -> Self

struct Bag:
    total: int

extend Bag implements Gather[int]:
    [I: needs Iterator[Item = int]]
    static fn gather(owns it: I) -> Bag:
        t = 0
        for x in it:
            t += x
        return Bag(total = t)

struct Count:
    n: int
    stop: int

extend Count implements Iterator:
    type Item = int

    fn next() -> Option[int]:
        if this.n >= this.stop:
            return None
        this.n += 1
        return Some(this.n)

[T, C: needs Gather[T], I: needs Iterator[Item = T]]
fn build(owns it: I) -> C:
    return C.gather(it)

fn main():
    b: Bag = build[int, Bag, Count](Count(n = 0, stop = 3))
    println(b.total)
