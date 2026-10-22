interface Shape:
    fn area(self) -> float

struct Circle implements Shape:
    r: float

    fn area(self) -> float:
        return 3.14 * self.r * self.r

fn describe(s: Shape) -> float:
    return s.area()

fn main():
    println(describe(Circle(r=2.0)))
