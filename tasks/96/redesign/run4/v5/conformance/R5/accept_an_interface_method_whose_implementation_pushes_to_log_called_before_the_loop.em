#$ test: run-pass
#$ rules: R5, R8, R14
#$ stdout: start 4.0
#$ stdout: area 4.0
## R5, R8 (c), verdict 86: the nearest program that is allowed: `s.area()`, which may run `Square.area`
## and push to `LOG`, runs before the loop over `LOG`.

static mut LOG: Array[String] = []

interface Shape:
    fn area() -> float

class Square:
    side: float = 2.0

extend Square implements Shape:
    fn area() -> float:
        LOG.push(String("area"))
        return this.side * this.side

fn report(s: dyn Shape):
    a = s.area()
    for line in LOG:
        println(line, a)

fn main():
    LOG.push(String("start"))
    sq = Square()
    report(sq)
