interface Shape:
    fn area() -> float

struct Circle implements Shape:
    r: float

    fn area() -> float:
        return 3.14 * this.r * this.r

fn describe(s: Shape) -> float:
    return s.area()

fn main():
    println(describe(Circle(r=2.0)))
