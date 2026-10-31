interface Shape:
    fn area() -> float

class Circle implements Shape:
    r: float

    fn area() -> float:
        return 3.0 * this.r * this.r

class Square implements Shape:
    s: float

    fn area() -> float:
        return this.s * this.s

fn main():
    shapes: Array[Shape] = []
    for i in 0..1000000:
        if i % 2 == 0:
            shapes.push(Circle(1.0))
        else:
            shapes.push(Square(2.0))
    total = 0.0
    for round in 0..20:
        for s in shapes:
            total = total + s.area()
    println(total as int)
