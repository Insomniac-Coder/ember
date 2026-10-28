#$ test: compile-fail
#$ rules: BRW-18, CTL-2
#$ help: move `d.log.push(String("open"))` below line 23
## design rules: R8, R6
## R8 (c), R6, verdict 16: a generator is a walk of what it views however it walks it; one that walks
## by position (`while i < this.log.len():`) is refused the same way, with no way out by position.
## AN-77, the v3 note's point 1.

class Door:
    log: Array[String] = []

    gen fn replay() -> Generator[int] from log:
        i = 0
        while i < this.log.len():
            yield this.log[i].len()
            i += 1

fn main():
    d = Door()
    d.log.push(String("shut"))
    g = d.replay()
    println(g.next().unwrap())
    d.log.push(String("open"))    #$ error[E3020]: `d.log.push(String("open"))` (line 22) changes `d.log`, which `g` walks (`g = d.replay()`, line 20), and `g` is used again on line 23; push after `g`'s last use: move `d.log.push(String("open"))` below line 23
    println(g.next().unwrap())
