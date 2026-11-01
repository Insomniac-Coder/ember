#$ test: run-pass
#$ rules: TYP-18, TYP-17
#$ stdout: 6
#$ stdout: 6
#$ stdout: Some(1)
# D-438 — a parameter no argument names but a solved one's binding does is
# what that type says: `T` in `I: Iterator[Item = T]` once `I` is known, at a
# call (was E2060) and where an extension is matched (it never applied).

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
    return C.gather(owns it)

[I]
struct Holder:
    it: I

extend[T: needs Copy, I: needs Iterator[Item = T]] Holder[I]:
    fn first() -> Option[T]:
        return this.it.next()

fn main():
    b: Bag = build(owns Count(n = 0, stop = 3))
    println(b.total)
    c: Bag = build(owns (1..4).iter())
    println(c.total)
    xs = [1, 2]
    h = Holder(xs.iter().copy_each())
    println(h.first())
