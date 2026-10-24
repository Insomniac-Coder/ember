# A13: the `if`-binding a Python or Rust programmer tries first for a checked downcast.
open class Shape:
    virtual fn area(self) -> f64:
        return 0.0

class Circle(Shape):
    r: f64 = 1.0

fn total_radius(shapes: Array[Shape]) -> f64:
    t = 0.0
    for s in shapes:
        if c = cast s if Circle:
            t += c.r
    return t

fn main():
    println(total_radius([Circle()]))
