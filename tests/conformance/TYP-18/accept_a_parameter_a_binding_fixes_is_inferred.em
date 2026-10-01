#$ test: run-pass
#$ rules: TYP-18, TYP-17
#$ stdout: 6
#$ stdout: 6
#$ stdout: Some(1)
# D-438 — a parameter no argument names but a solved one's binding does is
# what that type says: `T` in `I: Iterator[Item = T]` once `I` is known, at a
# call (was E2060) and where an extension is matched (it never applied).

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

struct Holder[I]:
    it: I

extend[T: Copy, I: Iterator[Item = T]] Holder[I]:
    fn first(mut self) -> Option[T]:
        return self.it.next()

fn main():
    b: Bag = build(Count(n = 0, stop = 3))
    println(b.total)
    c: Bag = build((1..4).iter())
    println(c.total)
    xs = [1, 2]
    h = Holder(xs.iter().copied())
    println(h.first())
