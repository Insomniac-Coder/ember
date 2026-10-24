# A13: Python's isinstance over a list of shapes (class objects), with today's `cast ... if`.
open class Shape:
    virtual fn area(self) -> f64:
        return 0.0

class Circle(Shape):
    r: f64 = 1.0
    override fn area(self) -> f64:
        return 3.0 * self.r * self.r

class Square(Shape):
    side: f64 = 2.0
    override fn area(self) -> f64:
        return self.side * self.side

fn total_radius(shapes: Array[Shape]) -> f64:
    t = 0.0
    for s in shapes:
        match cast s if Circle:
            Some(c):
                t += c.r
            None:
                pass
    return t

fn main():
    shapes: Array[Shape] = [Circle(), Square()]
    println(total_radius(shapes))
