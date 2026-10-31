#$ test: run-pass
#$ rules: IFC-4, TYP-17, TYP-18, CLO-14, GRM-8c
#$ profiles: debug, development, release
#$ stdout: 12
# D-540: the positional argument introduces a projection before equality and
# callable payloads are resolved. All three references must name the same type.

interface Source:
    type Item: needs Copy
    fn get() -> Item

interface Build[T]:
    type Stored
    static fn make(value: T) -> Self
    fn get() -> Stored

struct Numbers:
    value: int

extend Numbers implements Source:
    type Item = int
    fn get() -> int:
        return this.value

struct StoredInt:
    value: int

extend StoredInt implements Build[int]:
    type Stored = int
    static fn make(value: int) -> StoredInt:
        return StoredInt(value)
    fn get() -> int:
        return this.value

[S: needs Source, C: needs Build[S.Item, Stored = S.Item], F: needs fn(S.Item) -> int]
fn through(source: S, transform: F) -> int:
    stored = C.make(source.get())
    return transform(stored.get())

type Transform = fn(int) -> int

fn plus_five(value: int) -> int:
    return value + 5

fn main():
    println(through[Numbers, StoredInt, Transform](Numbers(7), plus_five))
