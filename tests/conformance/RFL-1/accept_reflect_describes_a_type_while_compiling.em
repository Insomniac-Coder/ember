#$ test: run-pass
#$ rules: RFL-1
#$ stdout: Point true 16 16 2
#$ stdout: x f64 0
#$ stdout: y f64 8
#$ stdout: derive 0 align 1
#$ stdout: Shape 3 true
#$ stdout: Dot 0 0
#$ stdout: Circle 1 1 8
#$ stdout: Rect 2 2 12
#$ stdout: Node Class 2 value
#$ stdout: (i64, str) Tuple 2
#$ stdout: f64 Float 8
## `[RFL-1]` — `reflect[T]()` describes a type while compiling: its name, kind, size and
## alignment, its fields (name, type, offset, attributes), its variants and its attributes. A
## description owns arrays, so it is kept in a `static`, or read in `comptime(…)`.

from std.ser import reflect, TypeDesc, TypeKind, AttrValue

@derive(Debug)
@align(16)
struct Point:
    x: f64
    y: f64

enum Shape:
    Dot
    Circle(f64)
    Rect(w: f32, h: f32)

class Node:
    value: int
    name: String

static POINT: TypeDesc = reflect[Point]()
static SHAPE: TypeDesc = reflect[Shape]()
static NODE: TypeDesc = reflect[Node]()
static PAIR: TypeDesc = reflect[(int, str)]()

fn main():
    println(POINT.name, POINT.kind == TypeKind.Struct, POINT.size, POINT.align, len(POINT.fields))
    for f in POINT.fields:
        println(f.name, f.type_name, f.offset)
    derive = ref POINT.attributes[0]
    align = ref POINT.attributes[1]
    println(derive.name, derive.args[0].0, align.name, len(align.args))
    println(SHAPE.name, len(SHAPE.variants), SHAPE.kind == TypeKind.Enum)
    for v in SHAPE.variants:
        if len(v.fields) == 0:
            println(v.name, v.discriminant, 0)
        else:
            println(v.name, v.discriminant, len(v.fields), v.fields[len(v.fields) - 1].offset)
    println(NODE.name, NODE.kind, len(NODE.fields), NODE.fields[0].name)
    println(PAIR.name, PAIR.kind, len(PAIR.fields))
    println(comptime(reflect[f64]().name), comptime(reflect[f64]().kind), comptime(reflect[f64]().size))
    comptime:
        assert(reflect[Point]().size == 16, "Point is 16 bytes")
