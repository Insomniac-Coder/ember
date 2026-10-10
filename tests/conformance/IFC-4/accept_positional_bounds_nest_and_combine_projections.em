#$ test: run-pass
#$ rules: IFC-4, TYP-17, TYP-18
#$ profiles: debug, release, shipping
#$ stdout: 11 word
#$ stdout: 13 pair
# D-540: positional bound arguments find projections below constructors and
# keep projections of two different parameters in distinct hidden slots.

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

struct IntOption:
    value: Option[int]

extend IntOption implements Build[Option[int]]:
    fn make(value: Option[int]) -> IntOption:
        return IntOption(value)

struct TextOption:
    value: Option[str]

extend TextOption implements Build[Option[str]]:
    fn make(value: Option[str]) -> TextOption:
        return TextOption(value)

struct Pair:
    value: (int, str)

extend Pair implements Build[(int, str)]:
    fn make(value: (int, str)) -> Pair:
        return Pair(value)

fn optional[S: Source, C: Build[Option[S.Item]]](source: S) -> C:
    return C.make(Some(source.get()))

fn pair[L: Source, R: Source, C: Build[(L.Item, R.Item)]](left: L, right: R) -> C:
    return C.make((left.get(), right.get()))

fn main():
    number = optional[IntSource, IntOption](IntSource(11))
    text = optional[TextSource, TextOption](TextSource("word"))
    both = pair[IntSource, TextSource, Pair](IntSource(13), TextSource("pair"))
    println(number.value.unwrap(), text.value.unwrap())
    println(both.value.0, both.value.1)
