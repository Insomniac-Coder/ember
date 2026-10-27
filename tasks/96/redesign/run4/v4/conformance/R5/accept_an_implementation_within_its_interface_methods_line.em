#$ test: run-pass
#$ rules: R5
#$ stdout: 4.0 1
## R5: the interface method's first line says `changes this`, so an implementation that changes one
## field keeps to it.

interface Shape:
    fn area() changes this -> float

class Square:
    side: float = 2.0
    calls: int = 0

extend Square implements Shape:
    fn area() changes calls -> float:
        this.calls += 1
        return this.side * this.side

fn main():
    s = Square()
    a = s.area()
    println(a, s.calls)
