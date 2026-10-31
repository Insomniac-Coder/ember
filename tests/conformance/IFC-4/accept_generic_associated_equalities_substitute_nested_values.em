#$ test: run-pass
#$ rules: IFC-4, TYP-17, GRM-8c
#$ stdout: 7 text
#$ stdout: 11 word
#$ stdout: true true
# Each Outer instance substitutes its argument inside the associated equality
# Option[T]. Fixed and opaque callers must agree without mixing the instances.

interface Element:
    type Item
    fn get() -> Item

interface Outer[T]:
    type Value: needs Element[Item = Option[T]]
    fn value() -> Value

@derive(Copy)
struct IntElement:
    item: Option[int]

extend IntElement implements Element:
    type Item = Option[int]
    fn get() -> Option[int]:
        return this.item

@derive(Copy)
struct TextElement:
    item: Option[StrView]

extend TextElement implements Element:
    type Item = Option[StrView]
    fn get() -> Option[StrView]:
        return this.item

struct IntOwner:
    item: Option[int]

extend IntOwner implements Outer[int]:
    type Value = IntElement
    fn value() -> IntElement:
        return IntElement(this.item)

struct TextOwner:
    item: Option[StrView]

extend TextOwner implements Outer[StrView]:
    type Value = TextElement
    fn value() -> TextElement:
        return TextElement(this.item)

[T, O: needs Outer[T]]
fn through(o: O) -> Option[T]:
    return o.value().get()

[O: needs Outer[int]]
fn integers(o: O) -> Option[int]:
    return o.value().get()

[O: needs Outer[StrView]]
fn texts(o: O) -> Option[StrView]:
    return o.value().get()

fn main():
    println(through[int, IntOwner](IntOwner(Some(7))).unwrap(),
            through[StrView, TextOwner](TextOwner(Some("text"))).unwrap())
    println(integers(IntOwner(Some(11))).unwrap(), texts(TextOwner(Some("word"))).unwrap())
    println(through[int, IntOwner](IntOwner(None)).is_none(),
            through[StrView, TextOwner](TextOwner(None)).is_none())
