#$ test: run-pass
#$ rules: R6, R8
#$ stdout: 4
#$ 2
## R6, verdict 16: the nearest allowed program: the push comes after `g`'s last use, and `d.angle`,
## which `from log` leaves out, may change while `g` walks (AN-77).

class Door:
    angle: float = 0.0
    log: Array[String] = []

    gen fn replay() -> Generator[int] from log:
        for entry in this.log:
            yield entry.len()

fn main():
    d = Door()
    d.log.push(String("open"))
    g = d.replay()
    d.angle = 90.0
    for n in g:
        println(n)
    d.log.push(String("x"))
    println(d.log.len())
