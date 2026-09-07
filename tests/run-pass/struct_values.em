#$ test: run-pass
#$ rules: STR-1, TYP-11, LEX-16

struct Point:
    x: f32
    y: f32

struct Line:
    start: Point
    end: Point

fn midpoint(l: Line) -> Point:
    return Point((l.start.x + l.end.x) * 0.5, (l.start.y + l.end.y) * 0.5)

fn main():
    l = Line(Point(0, 0), Point(4, 10))
    m = midpoint(l)
    println(m.x)
    println(m.y)
#$ stdout: 2
#$ 5
