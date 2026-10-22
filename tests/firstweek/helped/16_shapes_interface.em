# Shapes behind an interface: circles, rectangles and triangles share area and perimeter, then get listed, summed and sorted.

import std.math

interface Shape:
    fn area(self) -> float
    fn perimeter(self) -> float
    fn name(self) -> String

struct Circle:
    radius: float

struct Rectangle:
    width: float
    height: float

struct Triangle:
    a: float
    b: float
    c: float

extend Circle implements Shape:
    fn area(self) -> float:
        return math.PI * self.radius ** 2
    fn perimeter(self) -> float:
        return 2.0 * math.PI * self.radius
    fn name(self) -> String:
        return f"circle r={self.radius}"

extend Rectangle implements Shape:
    fn area(self) -> float:
        return self.width * self.height
    fn perimeter(self) -> float:
        return 2.0 * (self.width + self.height)
    fn name(self) -> String:
        if self.width == self.height:
            return f"square {self.width}"
        return f"rectangle {self.width}x{self.height}"

extend Triangle implements Shape:
    fn area(self) -> float:
        s = self.perimeter() / 2.0
        return (s * (s - self.a) * (s - self.b) * (s - self.c)).sqrt()
    fn perimeter(self) -> float:
        return self.a + self.b + self.c
    fn name(self) -> String:
        return f"triangle {self.a}/{self.b}/{self.c}"

fn describe(s: Box[dyn Shape]):
    println(f"{s.name():<20} area {s.area():8.2f}  perimeter {s.perimeter():8.2f}")

fn total_area(shapes: Array[Box[dyn Shape]]) -> float:
    return sum(s.area() for s in shapes)

fn main():
    shapes: Array[Box[dyn Shape]] = [
        Box(Circle(radius=1.5)),
        Box(Rectangle(width=2.0, height=3.0)),
        Box(Triangle(a=3.0, b=4.0, c=5.0)),
        Box(Rectangle(width=2.5, height=2.5)),
        Box(Circle(radius=0.5)),
    ]
    for s in shapes:
        describe(s)
    println(f"total area: {total_area(shapes):.3f}")

    shapes.sort_by_key(fn(s) => s.area())
    println("smallest to largest:")
    for s in shapes:
        println(f"  {s.name()}")

    big = shapes.iter().filter(fn(s) => s.area() > 5.0).count()
    println(f"{big} shapes are bigger than 5")
