#$ test: run-pass
#$ rules: STR-5, TYP-36, STD-9, LEX-19
#$ stdout: Point(x=1, y=2.5)
#$ Line(a=Point(x=1, y=2.5), name='l')
#$ Shape.Circle(1.5) Shape.Rect(2, 3) Shape.Empty
#$ Slow
#$ [Mode.Fast, Mode.Slow]
#$ Slow Mode.Slow Point(x=1, y=2.5) [  Slow]
#$ Pair(left=1, right=2)
#$ Tree(value=1, kids=[Tree(value=2, kids=[])])
#$ Some(Point(x=1, y=2.5))
# `[STR-5]` — structs and enums have `Debug` field-wise with nothing written,
# in `[TYP-36]`'s forms: `Point(x=1, y=2.5)`, `Shape.Circle(1.5)`, `Mode.Fast`.
# A struct or payload enum displays as its `Debug`; a unit-only enum displays
# as its variant name, and `!r` asks for the `Debug`. A generic struct shows
# its name alone, and a recursive one prints to any depth.

struct Point:
    x: int
    y: float

struct Line:
    a: Point
    name: String

enum Shape:
    Circle(float)
    Rect(int, int)
    Empty

enum Mode:
    Fast
    Slow

struct Pair[T]:
    left: T
    right: T

struct Tree:
    value: int
    kids: Array[Tree]

fn main():
    p = Point(x=1, y=2.5)
    println(p)
    println(Line(a=p.clone(), name="l"))
    println(Shape.Circle(1.5), Shape.Rect(2, 3), Shape.Empty)
    m = Mode.Slow
    println(m)
    println([Mode.Fast, m])
    println(f"{m} {m!r} {p} [{m!s:>6}]")
    println(Pair[int](left=1, right=2))
    println(Tree(value=1, kids=[Tree(value=2, kids=[])]))
    o: Option[Point] = Some(p.clone())
    println(o)
