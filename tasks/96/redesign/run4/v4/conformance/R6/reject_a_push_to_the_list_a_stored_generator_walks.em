#$ test: compile-fail
#$ rules: R6, R8
#$ help: d.log.push(String("x"))
## R6, verdict 16: a generator method's result is a view that walks what it views (`from log`), so the
## list does not change while the generator will still be used (R8 (c); AN-77, E3020).

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
    d.log.push(String("x"))    #$ error[E3020]: `d.log.push(String("x"))` (line 19) changes `d.log`, which `g` walks (`g = d.replay()`, line 18), and `g` is used again on line 20; push after `g`'s last use: move `d.log.push(String("x"))` below line 21
    for n in g:
        println(n)
