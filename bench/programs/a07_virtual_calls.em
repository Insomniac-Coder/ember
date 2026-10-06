interface Shape:
    fn area(self) -> float

class Circle implements Shape:
    r: float

    fn area(self) -> float:
        return 3.0 * self.r * self.r

class Square implements Shape:
    s: float

    fn area(self) -> float:
        return self.s * self.s

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
