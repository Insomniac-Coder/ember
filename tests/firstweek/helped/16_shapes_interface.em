# Shapes behind an interface: circles, rectangles and triangles share area and perimeter, then get listed, summed and sorted.

import std.math

interface Shape:
    fn area() -> float
    fn perimeter() -> float
    fn name() -> String

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
    fn area() -> float:
        return math.PI * this.radius ** 2
    fn perimeter() -> float:
        return 2.0 * math.PI * this.radius
    fn name() -> String:
        return f"circle r={this.radius}"

extend Rectangle implements Shape:
    fn area() -> float:
        return this.width * this.height
    fn perimeter() -> float:
        return 2.0 * (this.width + this.height)
    fn name() -> String:
        if this.width == this.height:
            return f"square {this.width}"
        return f"rectangle {this.width}x{this.height}"

extend Triangle implements Shape:
    fn area() -> float:
        s = this.perimeter() / 2.0
        return (s * (s - this.a) * (s - this.b) * (s - this.c)).sqrt()
    fn perimeter() -> float:
        return this.a + this.b + this.c
    fn name() -> String:
        return f"triangle {this.a}/{this.b}/{this.c}"

fn describe(s: ref dyn Shape):
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

    shapes.sort_by_key(owns fn(s) => s.area())
    println("smallest to largest:")
    for s in shapes:
        println(f"  {s.name()}")

    big = [s for s in owns shapes if s.area() > 5.0]
    println(f"{big.len()} shapes are bigger than 5")
