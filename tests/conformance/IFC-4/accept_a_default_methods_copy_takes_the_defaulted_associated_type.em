#$ test: run-pass
#$ rules: IFC-4
#$ stdout: 6 4 18000000000000000000 9000000000000000001 10
# `[IFC-4]` (ODR-049) — an implementation that leaves an associated type to
# the interface's default (`A`, whose `Count` is the default `int`) gets each
# default method with that type in its signature as well as in its body. The
# copy's `-> Count` stayed `Self.Count` while its body's `self.size()` was
# the default, `E2020` (D-516). An implementation that states its own (`B`,
# `u64`) and a generic caller (`total`) see theirs.

interface Counting: Copy + Ord:
    fn of(n: int) -> Self
    fn plus(self, other: Self) -> Self

extend int implements Counting:
    fn of(n: int) -> int:
        return n
    fn plus(self, other: int) -> int:
        return self + other

extend u64 implements Counting:
    fn of(n: int) -> u64:
        return n as u64
    fn plus(self, other: u64) -> u64:
        return self + other

interface Thing:
    type Count: Counting = int
    fn size(self) -> Count
    fn twice(self) -> Count:
        return self.size().plus(self.size())
    fn one_more(self) -> Count:
        return self.size().plus(Count.of(1))

struct A:
    n: int

extend A implements Thing:
    fn size(self) -> int:
        return self.n

struct B:
    n: u64

extend B implements Thing:
    type Count = u64
    fn size(self) -> u64:
        return self.n

fn total[T: Thing](t: T) -> T.Count:
    return t.twice()

fn main():
    a = A(3)
    b = B(9000000000000000000 as u64)
    println(a.twice(), a.one_more(), b.twice(), b.one_more(), total(A(5)))
