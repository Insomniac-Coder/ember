#$ test: run-pass
#$ rules: IFC-4, TYP-17, TYP-18
#$ profiles: debug, release, shipping
#$ stdout: 17
# A method's hidden projection is rebased when its owner's type parameter is
# substituted. This case contains no positional projection bound.

interface Source:
    type Item: Copy
    fn get(self) -> Item

struct Numbers:
    value: int

extend Numbers implements Source:
    type Item = int
    fn get(self) -> int:
        return self.value

struct Holder[T]:
    marker: T

    fn item[S: Source](self, source: S) -> S.Item:
        return source.get()

fn main():
    holder = Holder[str]("owner")
    println(holder.item[Numbers](Numbers(17)))
