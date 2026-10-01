#$ test: run-pass
#$ rules: IFC-4, TYP-17
#$ stdout: 6
# D-437 (2) — an instance of a generic interface over the caller's own
# parameter (`Gather[T]`) numbers its method's own parameters after the
# caller's, so `C.gather(it)` no longer reads the method's `I` in the
# caller's slot (was E2040, "`I`'s `Item` for `Iterator` is `T`, but `I`'s bound
# needs `I`").

interface Gather[T]:
    fn gather[I: Iterator[Item = T]](owned it: I) -> Self

struct Bag:
    total: int

extend Bag implements Gather[int]:
    fn gather[I: Iterator[Item = int]](owned it: I) -> Bag:
        t = 0
        for x in it:
            t += x
        return Bag(total = t)

struct Count:
    n: int
    stop: int

extend Count implements Iterator:
    type Item = int

    fn next(mut self) -> Option[int]:
        if self.n >= self.stop:
            return None
        self.n += 1
        return Some(self.n)

fn build[T, C: Gather[T], I: Iterator[Item = T]](owned it: I) -> C:
    return C.gather(it)

fn main():
    b: Bag = build[int, Bag, Count](Count(n = 0, stop = 3))
    println(b.total)
