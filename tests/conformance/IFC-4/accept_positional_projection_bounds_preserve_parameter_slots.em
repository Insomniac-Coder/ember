#$ test: run-pass
#$ rules: IFC-4, TYP-17, TYP-18
#$ profiles: debug, release, shipping
#$ stdout: 7 true
#$ stdout: owner 9 2.5
# D-540: introducing S.Item must not reuse U's later written slot. A generic
# method also keeps its enclosing type's prefix distinct from both of them.

interface Source:
    type Item: Copy
    fn get(self) -> Item

interface Build[T]:
    fn make(value: T) -> Self

struct IntSource:
    value: int

extend IntSource implements Source:
    type Item = int
    fn get(self) -> int:
        return self.value

struct IntValue:
    value: int

extend IntValue implements Build[int]:
    fn make(value: int) -> IntValue:
        return IntValue(value)

fn later[S: Source, C: Build[S.Item], U: Copy](source: S, extra: U) -> (C, U):
    return (C.make(source.get()), extra)

struct Holder[T: Copy]:
    marker: T

    fn assemble[S: Source, C: Build[S.Item], U: Copy](self, source: S, extra: U) -> (T, C, U):
        return (self.marker, C.make(source.get()), extra)

fn main():
    a = later[IntSource, IntValue, bool](IntSource(7), true)
    holder = Holder[str]("owner")
    b = holder.assemble[IntSource, IntValue, float](IntSource(9), 2.5)
    println(a.0.value, a.1)
    println(b.0, b.1.value, b.2)
