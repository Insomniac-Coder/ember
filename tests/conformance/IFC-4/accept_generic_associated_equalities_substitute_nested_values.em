#$ test: run-pass
#$ rules: IFC-4, TYP-17, GRM-8c
#$ stdout: 7 text
#$ stdout: 11 word
#$ stdout: true true
# Each Outer instance substitutes its argument inside the associated equality
# Option[T]. Fixed and opaque callers must agree without mixing the instances.

interface Element:
    type Item
    fn get(self) -> Item

interface Outer[T]:
    type Value: Element[Item = Option[T]]
    fn value(self) -> Value

@derive(Copy)
struct IntElement:
    item: Option[int]

extend IntElement implements Element:
    type Item = Option[int]
    fn get(self) -> Option[int]:
        return self.item

@derive(Copy)
struct TextElement:
    item: Option[str]

extend TextElement implements Element:
    type Item = Option[str]
    fn get(self) -> Option[str]:
        return self.item

struct IntOwner:
    item: Option[int]

extend IntOwner implements Outer[int]:
    type Value = IntElement
    fn value(self) -> IntElement:
        return IntElement(self.item)

struct TextOwner:
    item: Option[str]

extend TextOwner implements Outer[str]:
    type Value = TextElement
    fn value(self) -> TextElement:
        return TextElement(self.item)

fn through[T, O: Outer[T]](o: O) -> Option[T]:
    return o.value().get()

fn integers[O: Outer[int]](o: O) -> Option[int]:
    return o.value().get()

fn texts[O: Outer[str]](o: O) -> Option[str]:
    return o.value().get()

fn main():
    println(through[int, IntOwner](IntOwner(Some(7))).unwrap(),
            through[str, TextOwner](TextOwner(Some("text"))).unwrap())
    println(integers(IntOwner(Some(11))).unwrap(), texts(TextOwner(Some("word"))).unwrap())
    println(through[int, IntOwner](IntOwner(None)).is_none(),
            through[str, TextOwner](TextOwner(None)).is_none())
