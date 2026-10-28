#$ test: run-pass
#$ rules: FN-4
#$ stdout: 4.0 1
## design rules: R5
## R5, verdict 86: an implementation of an interface method may change its object (`this.calls += 1`);
## no first line says so, and a call through the interface counts what every implementation changes.

interface Shape:
    fn area() -> float

class Square:
    side: float = 2.0
    calls: int = 0

extend Square implements Shape:
    fn area() -> float:
        this.calls += 1
        return this.side * this.side

fn main():
    s = Square()
    a = s.area()
    println(a, s.calls)
