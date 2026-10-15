#$ test: run-pass
#$ rules: EFF-8
#$ stdout: 9
# `[EFF-8]` — an override with none of the effects its base method forbids keeps the contract.

open class Shape:
    @noio
    virtual fn area(self) -> int:
        return 0

class Square(Shape):
    side: int = 3

    override fn area(self) -> int:
        return self.side * self.side

fn main():
    s: Shape = Square()
    println(s.area())
