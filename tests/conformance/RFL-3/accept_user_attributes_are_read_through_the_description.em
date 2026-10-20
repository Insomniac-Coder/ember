#$ test: run-pass
#$ rules: RFL-3, RFL-2
#$ stdout: Range min 0.0 max 1.0
#$ stdout: Label text speed
#$ stdout: Tag 0 7
## `[RFL-3]` — a struct marked `@attribute` is written as an attribute on the fields and
## declarations of `@reflect` types, checked exactly as its constructor call, and its value is read
## through the type's description.

from std.ser import type_info, AttrValue

@attribute
struct Range:
    min: float
    max: float

@attribute
struct Label:
    text: str

@attribute
struct Tag:
    id: int

const TOP: float = 1.0

@reflect
@Tag(7)
struct Car:
    @Range(min=0.0, max=TOP)
    @Label("speed")
    speed: float
    wheels: int

fn show(value: AttrValue) -> String:
    match value:
        AttrValue.Float(x):
            return f"{x}"
        AttrValue.Int(n):
            return f"{n}"
        AttrValue.Str(s):
            return s.clone()
        _:
            return String.from("?")

fn main():
    info = type_info[Car]()
    speed = ref info.fields[0]
    range = ref speed.attributes[0]
    println(range.name, range.args[0].0, show(range.args[0].1), range.args[1].0, show(range.args[1].1))
    label = ref speed.attributes[1]
    println(label.name, label.args[0].0, show(label.args[0].1))
    tag = ref info.attributes[1]
    println(tag.name, len(info.fields[1].attributes), show(tag.args[0].1))
