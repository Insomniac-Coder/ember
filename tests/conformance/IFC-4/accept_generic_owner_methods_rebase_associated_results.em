#$ test: run-pass
#$ rules: IFC-4, TYP-17, TYP-18
#$ profiles: debug, development, release
#$ stdout: 17
# A method's hidden projection is rebased when its owner's type parameter is
# substituted. This case contains no positional projection bound.

interface Source:
    type Item: needs Copy
    fn get() -> Item

struct Numbers:
    value: int

extend Numbers implements Source:
    type Item = int
    fn get() -> int:
        return this.value

[T]
struct Holder:
    marker: T

    [S: needs Source]
    fn item(source: S) -> S.Item:
        return source.get()

fn main():
    holder = Holder[StrView]("owner")
    println(holder.item[Numbers](Numbers(17)))
