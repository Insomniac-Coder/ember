#$ test: run-pass
#$ rules: IFC-4, TYP-17, TYP-18
#$ profiles: debug, development, release
#$ stdout: 7 true
#$ stdout: owner 9 2.5
# D-540: introducing S.Item must not reuse U's later written slot. A generic
# method also keeps its enclosing type's prefix distinct from both of them.

interface Source:
    type Item: needs Copy
    fn get() -> Item

interface Build[T]:
    static fn make(value: T) -> Self

struct IntSource:
    value: int

extend IntSource implements Source:
    type Item = int
    fn get() -> int:
        return this.value

struct IntValue:
    value: int

extend IntValue implements Build[int]:
    static fn make(value: int) -> IntValue:
        return IntValue(value)

[S: needs Source, C: needs Build[S.Item], U: needs Copy]
fn later(source: S, extra: U) -> (C, U):
    return (C.make(source.get()), extra)

[T: needs Copy]
struct Holder:
    marker: T

    [S: needs Source, C: needs Build[S.Item], U: needs Copy]
    fn assemble(source: S, extra: U) -> (T, C, U):
        return (this.marker, C.make(source.get()), extra)

fn main():
    a = later[IntSource, IntValue, bool](IntSource(7), true)
    holder = Holder[StrView]("owner")
    b = holder.assemble[IntSource, IntValue, float](IntSource(9), 2.5)
    println(a.0.value, a.1)
    println(b.0, b.1.value, b.2)
