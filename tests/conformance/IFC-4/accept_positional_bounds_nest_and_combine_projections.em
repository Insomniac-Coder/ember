#$ test: run-pass
#$ rules: IFC-4, TYP-17, TYP-18
#$ profiles: debug, development, release
#$ stdout: 11 word
#$ stdout: 13 pair
# D-540: positional bound arguments find projections below constructors and
# keep projections of two different parameters in distinct hidden slots.

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

struct IntOption:
    value: Option[int]

extend IntOption implements Build[Option[int]]:
    static fn make(value: Option[int]) -> IntOption:
        return IntOption(value)

struct TextOption:
    value: Option[StrView]

extend TextOption implements Build[Option[StrView]]:
    static fn make(value: Option[StrView]) -> TextOption:
        return TextOption(value)

struct Pair:
    value: (int, StrView)

extend Pair implements Build[(int, StrView)]:
    static fn make(value: (int, StrView)) -> Pair:
        return Pair(value)

[S: needs Source, C: needs Build[Option[S.Item]]]
fn optional(source: S) -> C:
    return C.make(Some(source.get()))

[L: needs Source, R: needs Source, C: needs Build[(L.Item, R.Item)]]
fn pair(left: L, right: R) -> C:
    return C.make((left.get(), right.get()))

fn main():
    number = optional[IntSource, IntOption](IntSource(11))
    text = optional[TextSource, TextOption](TextSource("word"))
    both = pair[IntSource, TextSource, Pair](IntSource(13), TextSource("pair"))
    println(number.value.unwrap(), text.value.unwrap())
    println(both.value.0, both.value.1)
