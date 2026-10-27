#$ test: run-pass
#$ rules: BRW-16, LT-1
#$ stdout: 4 90.0
## design rules: R8, R6
## R8 (c), R6: `from log` narrows what the generator walks to `d.log`, so another field of `d` may
## change while `g` will still be used. AN-77, the v3 note's point 2.

class Door:
    angle: float = 0.0
    log: Array[String] = []

    gen fn replay() -> Generator[int] from log:
        for entry in this.log:
            yield entry.len()

fn main():
    d = Door()
    d.log.push(String("shut"))
    g = d.replay()
    d.angle = 90.0
    println(g.next().unwrap(), d.angle)
