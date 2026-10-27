#$ test: compile-fail
#$ rules: BRW-18, CTL-2
#$ help: move `d.log.push(String("open"))` below line 21
## design rules: R8, R6
## R8 (c), verdict 16: a generator paused in the middle of a list is a loop over it; while `g` will
## still be used, `d.log` does not change. AN-77, point 6; the message of rules section 3.

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
    println(g.next().unwrap())
    d.log.push(String("open"))    #$ error[E3020]: `d.log.push(String("open"))` (line 20) changes `d.log`, which `g` walks (`g = d.replay()`, line 18), and `g` is used again on line 21; push after `g`'s last use: move `d.log.push(String("open"))` below line 21
    println(g.next().unwrap())
