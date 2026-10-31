#$ test: run-pass
#$ rules: IFC-4
#$ stdout: 6 4 18000000000000000000 9000000000000000001 10
# `[IFC-4]` (ODR-049) — an implementation that leaves an associated type to
# the interface's default (`A`, whose `Count` is the default `int`) gets each
# default method with that type in its signature as well as in its body. The
# copy's `-> Count` stayed `Self.Count` while its body's `self.size()` was
# the default, `E2020` (D-516). An implementation that states its own (`B`,
# `u64`) and a generic caller (`total`) see theirs.

interface Counting: needs (Copy, Ord):
    static fn of(n: int) -> Self
    fn plus(other: Self) -> Self

extend int implements Counting:
    static fn of(n: int) -> int:
        return n
    fn plus(other: int) -> int:
        return this + other

extend u64 implements Counting:
    static fn of(n: int) -> u64:
        return n as u64
    fn plus(other: u64) -> u64:
        return this + other

interface Thing:
    type Count: needs Counting = int
    fn size() -> Count
    fn twice() -> Count:
        return this.size().plus(this.size())
    fn one_more() -> Count:
        return this.size().plus(Count.of(1))

struct A:
    n: int

extend A implements Thing:
    fn size() -> int:
        return this.n

struct B:
    n: u64

extend B implements Thing:
    type Count = u64
    fn size() -> u64:
        return this.n

[T: needs Thing]
fn total(t: T) -> T.Count:
    return t.twice()

fn main():
    a = A(3)
    b = B(9000000000000000000 as u64)
    println(a.twice(), a.one_more(), b.twice(), b.one_more(), total(A(5)))
