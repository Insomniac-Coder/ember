#$ test: run-pass
#$ rules: ENM-1, ENM-2, ENM-3, TYP-12
#$ assert-c: contains("switch (")
#$ assert-c: contains("typedef uint8_t em_Dir")

enum Dir:
    North
    South
    East
    West

@repr(i16)
enum Level:
    Low = 10
    Mid = 20
    High = 30

enum Shape:
    Circle(f32)
    Rect(w: f32, h: f32)
    Empty

struct Point:
    x: i32
    y: i32

# One arm per variant, no guards: this is the shape that becomes a `switch`.
fn turn(d: Dir) -> i32:
    match d:
        Dir.North:
            return 0
        Dir.South:
            return 180
        Dir.East:
            return 90
        Dir.West:
            return 270

# Payloads are bound by position, and `[ENM-1]` allows the enum name to be
# left off when the scrutinee says which enum is meant.
fn area(s: Shape) -> f32:
    match s:
        Circle(r):
            return r * r
        Rect(w, h):
            return w * h
        Empty:
            return 0.0

# Guards fall through to the arms below when they fail.
fn describe(s: Shape) -> i32:
    match s:
        Circle(r) if r > 10.0:
            return 100
        Circle(r):
            return 1
        Rect(w, h) if w == h:
            return 200
        _:
            return 9

# `|` alternatives, and a binding that every alternative shares.
fn bucket(n: i32) -> i32:
    match n:
        0 | 1 | 2:
            return 10
        5:
            return 50
        x:
            return x * 2

# `[GRM-10]` — `=>` arms make the `match` an expression with a value.
fn degrees(d: Dir) -> i32:
    turned = match d:
        Dir.North => 0
        Dir.South => 180
        Dir.East => 90
        Dir.West => 270
    return turned + 1

# Patterns nest through tuples, structs and other enums.
fn nested(p: (Shape, i32)) -> i32:
    match p:
        (Circle(r), 0):
            return 1
        (Circle(r), n):
            return 2
        (Rect(w, h), _):
            return 3
        (Empty, _):
            return 4

fn origin(p: Point) -> i32:
    match p:
        Point(0, 0):
            return 1
        Point(x, 0):
            return 2
        Point(_, _):
            return 3

fn main():
    println(turn(Dir.South))
    println(turn(Dir.West))
    println(degrees(Dir.East))

    # `[ENM-3]` — a unit-only enum is its discriminant, and `as` reads it.
    println(Level.Mid as i32)
    println(Level.High as i32)

    println(area(Shape.Circle(3.0)))
    println(area(Shape.Rect(3.0, 4.0)))
    println(area(Shape.Empty))

    println(describe(Shape.Circle(20.0)))
    println(describe(Shape.Circle(1.0)))
    println(describe(Shape.Rect(3.0, 3.0)))
    println(describe(Shape.Empty))

    println(bucket(1))
    println(bucket(5))
    println(bucket(21))

    println(nested((Shape.Circle(1.0), 0)))
    println(nested((Shape.Circle(1.0), 9)))
    println(nested((Shape.Rect(1.0, 2.0), 0)))
    println(nested((Shape.Empty, 0)))

    println(origin(Point(0, 0)))
    println(origin(Point(4, 0)))
    println(origin(Point(4, 5)))

    # A named payload, given out of order.
    println(area(Shape.Rect(h=2.0, w=5.0)))
#$ stdout: 180
#$ 270
#$ 91
#$ 20
#$ 30
#$ 9
#$ 12
#$ 0
#$ 100
#$ 1
#$ 200
#$ 9
#$ 10
#$ 50
#$ 42
#$ 1
#$ 2
#$ 3
#$ 4
#$ 1
#$ 2
#$ 3
#$ 10
