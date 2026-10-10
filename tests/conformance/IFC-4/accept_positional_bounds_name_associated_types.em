#$ test: run-pass
#$ rules: IFC-4, TYP-17, TYP-18
#$ profiles: debug, release, shipping
#$ stdout: 7 text
# D-540: an interface's positional argument may be the associated type of an
# earlier parameter. Its hidden projection must exist before the bound is read.

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

struct TextSource:
    value: str

extend TextSource implements Source:
    type Item = str
    fn get(self) -> str:
        return self.value

struct IntValue:
    value: int

extend IntValue implements Build[int]:
    fn make(value: int) -> IntValue:
        return IntValue(value)

struct TextValue:
    value: str

extend TextValue implements Build[str]:
    fn make(value: str) -> TextValue:
        return TextValue(value)

fn build[S: Source, C: Build[S.Item]](source: S) -> C:
    return C.make(source.get())

fn main():
    number = build[IntSource, IntValue](IntSource(7))
    text = build[TextSource, TextValue](TextSource("text"))
    println(number.value, text.value)
