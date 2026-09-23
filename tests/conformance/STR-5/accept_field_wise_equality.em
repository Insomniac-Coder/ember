#$ test: run-pass
#$ rules: STR-5, TYP-37
#$ profiles: debug, release, shipping
#$ stdout: true false
#$ true false
#$ true false
#$ true false
#$ true false
#$ true false
# A struct or enum has `Eq` field-wise when every field does; tuples, fixed
# arrays and `Array`s compare element-wise. Nested aggregates compare through
# their own field-wise equality.

@derive(Copy)
struct Point:
    x: int
    y: int

struct Named:
    name: String
    at: Point
    tags: Array[String]

enum Shape:
    Dot
    Circle(Point, float)
    Label(String)

fn main():
    a = Point(x = 1, y = 2)
    println(a == Point(x = 1, y = 2), a == Point(x = 1, y = 3))
    n1 = Named(name = "n", at = a, tags = ["p", "q"])
    n2 = Named(name = "n", at = a, tags = ["p", "q"])
    n3 = Named(name = "n", at = a, tags = ["p"])
    println(n1 == n2, n1 == n3)
    println(Shape.Circle(a, 1.5) == Shape.Circle(a, 1.5), Shape.Circle(a, 1.5) == Shape.Dot)
    println(Shape.Label("x") != Shape.Label("y"), Shape.Dot != Shape.Dot)
    t = (1, "one", a)
    println(t == (1, "one", a), t == (1, "uno", a))
    fixed: [int; 3] = [1, 2, 3]
    other: [int; 3] = [1, 2, 4]
    println(fixed == fixed, fixed == other)
