#$ test: run-pass
#$ rules: IFC-4, TYP-17, TYP-18, CLO-14, GRM-8c
#$ profiles: debug, release, shipping
#$ stdout: 12
# D-540: the positional argument introduces a projection before equality and
# callable payloads are resolved. All three references must name the same type.

interface Source:
    type Item: Copy
    fn get(self) -> Item

interface Build[T]:
    type Stored
    fn make(value: T) -> Self
    fn get(self) -> Stored

struct Numbers:
    value: int

extend Numbers implements Source:
    type Item = int
    fn get(self) -> int:
        return self.value

struct StoredInt:
    value: int

extend StoredInt implements Build[int]:
    type Stored = int
    fn make(value: int) -> StoredInt:
        return StoredInt(value)
    fn get(self) -> int:
        return self.value

fn through[S: Source, C: Build[S.Item, Stored = S.Item], F: fn(S.Item) -> int](source: S, transform: F) -> int:
    stored = C.make(source.get())
    return transform(stored.get())

type Transform = fn(int) -> int

fn plus_five(value: int) -> int:
    return value + 5

fn main():
    println(through[Numbers, StoredInt, Transform](Numbers(7), plus_five))
