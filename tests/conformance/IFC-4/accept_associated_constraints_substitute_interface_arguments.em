#$ test: run-pass
#$ rules: IFC-4, TYP-16, TYP-17
#$ stdout: 7 11
# An instantiated interface's associated-type bounds use its actual arguments.
# Both the implementation check and calls through the projection see Tag[int]
# or Tag[str], never the declaration's unsubstituted Tag[T].

interface Tag[T]:
    fn tag() -> int

interface Outer[T]:
    type Member: needs Tag[T]
    fn member() -> Member

struct IntValue:
    n: int

extend IntValue implements Tag[int]:
    fn tag() -> int:
        return this.n

struct TextValue:
    n: int

extend TextValue implements Tag[StrView]:
    fn tag() -> int:
        return this.n

struct IntOwner:
    n: int

extend IntOwner implements Outer[int]:
    type Member = IntValue
    fn member() -> IntValue:
        return IntValue(this.n)

struct TextOwner:
    n: int

extend TextOwner implements Outer[StrView]:
    type Member = TextValue
    fn member() -> TextValue:
        return TextValue(this.n)

[T, O: needs Outer[T]]
fn read_tag(o: O) -> int:
    return o.member().tag()

fn main():
    println(read_tag[int, IntOwner](IntOwner(7)), read_tag[StrView, TextOwner](TextOwner(11)))
