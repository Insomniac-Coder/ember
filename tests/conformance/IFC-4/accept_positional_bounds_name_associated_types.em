#$ test: run-pass
#$ rules: IFC-4, TYP-17, TYP-18
#$ profiles: debug, development, release
#$ stdout: 7 text
# D-540: an interface's positional argument may be the associated type of an
# earlier parameter. Its hidden projection must exist before the bound is read.

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

struct TextSource:
    value: StrView

extend TextSource implements Source:
    type Item = StrView
    fn get() -> StrView:
        return this.value

struct IntValue:
    value: int

extend IntValue implements Build[int]:
    static fn make(value: int) -> IntValue:
        return IntValue(value)

struct TextValue:
    value: StrView

extend TextValue implements Build[StrView]:
    static fn make(value: StrView) -> TextValue:
        return TextValue(value)

[S: needs Source, C: needs Build[S.Item]]
fn build(source: S) -> C:
    return C.make(source.get())

fn main():
    number = build[IntSource, IntValue](IntSource(7))
    text = build[TextSource, TextValue](TextSource("text"))
    println(number.value, text.value)
