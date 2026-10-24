# A10: a list of shapes whose implementers are structs, written with no Box.
interface Shape:
    fn area(self) -> float

struct Square:
    side: float
    fn area(self) -> float:
        return self.side * self.side

struct Circle:
    r: float
    fn area(self) -> float:
        return 3.14159 * self.r * self.r

fn main():
    shapes: Array[Shape] = [Square(side=2.0), Circle(r=1.0)]
    s = 0.0
    for sh in shapes:
        s += sh.area()
    println(s)
