#$ test: compile-fail
#$ rules: R5, R12
#$ help: fn area() changes this -> float
## R5: a call through an interface method is checked against that method's first line, so an
## implementation may change no more than it says (E2113).

interface Shape:
    fn area() -> float

class Square:
    side: float = 2.0
    calls: int = 0

extend Square implements Shape:
    fn area() changes calls -> float:    #$ error[E2113]: `Square.area` changes `calls`, but it implements `Shape.area`, whose first line changes nothing, and callers of `Shape` rely on that; write `fn area() changes this -> float` in `Shape`
        this.calls += 1
        return this.side * this.side

fn main():
    s = Square()
    println(s.area())
