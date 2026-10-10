#$ test: run-pass
#$ rules: IFC-4, TYP-16, TYP-17
#$ stdout: 7 11
# An instantiated interface's associated-type bounds use its actual arguments.
# Both the implementation check and calls through the projection see Tag[int]
# or Tag[str], never the declaration's unsubstituted Tag[T].

interface Tag[T]:
    fn tag(self) -> int

interface Outer[T]:
    type Member: Tag[T]
    fn member(self) -> Member

struct IntValue:
    n: int

extend IntValue implements Tag[int]:
    fn tag(self) -> int:
        return self.n

struct TextValue:
    n: int

extend TextValue implements Tag[str]:
    fn tag(self) -> int:
        return self.n

struct IntOwner:
    n: int

extend IntOwner implements Outer[int]:
    type Member = IntValue
    fn member(self) -> IntValue:
        return IntValue(self.n)

struct TextOwner:
    n: int

extend TextOwner implements Outer[str]:
    type Member = TextValue
    fn member(self) -> TextValue:
        return TextValue(self.n)

fn read_tag[T, O: Outer[T]](o: O) -> int:
    return o.member().tag()

fn main():
    println(read_tag[int, IntOwner](IntOwner(7)), read_tag[str, TextOwner](TextOwner(11)))
