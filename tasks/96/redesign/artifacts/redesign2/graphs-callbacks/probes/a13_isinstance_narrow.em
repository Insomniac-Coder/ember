# A13, proposed: Python's isinstance, narrowing `s` inside the block (today: `match cast s if Circle`).
open class Shape:
    virtual fn area(self) -> f64:
        return 0.0

class Circle(Shape):
    r: f64 = 1.0

class Square(Shape):
    side: f64 = 2.0

fn total_radius(shapes: Array[Shape]) -> f64:
    t = 0.0
    for s in shapes:
        if isinstance(s, Circle):
            t += s.r
    return t

fn main():
    println(total_radius([Circle(), Square()]))
