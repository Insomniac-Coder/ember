#$ test: run-pass
#$ rules: DSP-3, DSP-2, TYP-22
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("->ti == &em_ti_") == 6
#$ stdout:
#$ 12 1009 6 1027
#$ 15
# `[DSP-3]` — an interface call on a class handle first compares the object's
# type information with the program's classes that implement the interface,
# when there are at most four, and calls each one's table entry directly (the
# C compiler can inline it); any other class still goes through the table
# search. `Shape` has three such classes: `Rect`, `Square`, which inherits
# the interface through `Rect` and overrides `area`, and `Tri`: three tests at
# each of the two `area` calls. The override still runs (`[DSP-2]`). `Named`
# has five, so its call only searches the table.

interface Shape:
    fn area(self) -> int

open class Rect implements Shape:
    w: int
    h: int

    virtual fn area(self) -> int:
        return self.w * self.h

class Square(Rect):
    override fn area(self) -> int:
        return self.w * self.w + 1000

class Tri implements Shape:
    b: int
    h: int

    fn area(self) -> int:
        return self.b * self.h // 2

interface Named:
    fn id(self) -> int

class N1 implements Named:
    fn id(self) -> int:
        return 1

class N2 implements Named:
    fn id(self) -> int:
        return 2

class N3 implements Named:
    fn id(self) -> int:
        return 3

class N4 implements Named:
    fn id(self) -> int:
        return 4

class N5 implements Named:
    fn id(self) -> int:
        return 5

fn main():
    shapes: Array[Shape] = []
    shapes.push(Rect(3, 4))
    square: Rect = Square(3, 3)
    shapes.push(square)
    shapes.push(Tri(3, 4))
    total = 0
    for s in shapes:
        total = total + s.area()
        print(s.area(), "")
    println(total)
    names: Array[Named] = [N1(), N2(), N3(), N4(), N5()]
    sum = 0
    for n in names:
        sum = sum + n.id()
    println(sum)
