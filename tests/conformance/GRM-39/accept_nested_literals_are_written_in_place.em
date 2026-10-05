#$ test: run-pass
#$ rules: GRM-39, ENM-3, STR-4
#$ profiles: debug, release, shipping
#$ stdout: 1 2
#$ stdout: 3 4
#$ stdout: 7 8 9
#$ stdout: 2 1
#$ stdout: 6 5
# G7-2 — nested literals written in place: an option of a tuple holding an
# option, a struct of two options, a struct inside an enum inside a tuple. A
# literal whose parts read the place it writes keeps its temporaries, since
# writing a first field would change what a later one reads.

struct Pair:
    a: Option[int]
    b: Option[int]

struct Point:
    x: int
    y: int

enum Shape:
    Dot(Point)
    Line(Point, Point)

fn main():
    t = Some((1, Some(2)))
    match t:
        Some((a, Some(b))):
            println(a, b)
        _:
            println("no")
    p = Pair(Some(3), Some(4))
    println(p.a.unwrap_or(0), p.b.unwrap_or(0))
    s = (Shape.Line(Point(7, 8), Point(9, 0)), 1)
    match s.0:
        Shape.Line(from, to):
            println(from.x, from.y, to.x)
        _:
            println("no")
    q = Point(1, 2)
    q = Point(q.y, q.x)
    println(q.x, q.y)
    r = Pair(Some(5), Some(6))
    r = Pair(r.b, r.a)
    println(r.a.unwrap_or(0), r.b.unwrap_or(0))
