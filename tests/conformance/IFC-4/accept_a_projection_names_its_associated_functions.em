#$ test: run-pass
#$ rules: IFC-4, TYP-17
#$ stdout: 12 40
# D-519, D-520 — inside a generic body, an associated type of a type parameter is named in an
# expression as in a type: `I.Size.of(2)` calls `Size`'s associated function (it was "cannot
# find `I`"). And a call through the parameter's bound reads the callee's `Self.Size` as the
# parameter's `I.Size` in its parameters too, not only its result: `it.skip_back(n)` takes an
# `I.Size`, and `I.grow(n)` takes and gives one.

interface Counter:
    static fn of(n: int) -> Self
    fn plus(other: Self) -> Self

interface Seq:
    type Size: needs Counter = int
    fn first() -> Size
    fn skip_back(n: Size) -> Size
    static fn grow(n: Size) -> Size

struct Tens:
    n: int

extend Tens implements Seq:
    fn first() -> int:
        return this.n

    fn skip_back(n: int) -> int:
        return this.n - n

    static fn grow(n: int) -> int:
        return n * 10

extend int implements Counter:
    static fn of(n: int) -> int:
        return n

    fn plus(other: int) -> int:
        return this + other

[I: needs Seq]
fn twelve(it: I) -> I.Size:
    return it.first().plus(I.Size.of(2))

[I: needs Seq]
fn forty(it: I) -> I.Size:
    return I.grow(it.skip_back(I.Size.of(6)))

fn main():
    println(twelve(Tens(10)), forty(Tens(10)))
