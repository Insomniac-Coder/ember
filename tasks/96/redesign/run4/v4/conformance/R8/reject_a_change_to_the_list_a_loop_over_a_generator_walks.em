#$ test: compile-fail
#$ rules: R8, R6
#$ help: d.log.clear()
## R8 (c), verdict 16: a `for` loop over a generator method walks what the generator views, here
## `d.log` (`from log`). AN-77, point 6.

class Door:
    log: Array[String] = []

    gen fn replay() -> Generator[int] from log:
        for entry in this.log:
            yield entry.len()

fn main():
    d = Door()
    d.log.push(String("shut"))
    for n in d.replay():
        d.log.clear()    #$ error[E3020]: this loop walks `d.log` through `d.replay()` (line 17; `replay` says `from log`), and `d.log.clear()` (line 18) changes it; move `d.log.clear()` below the loop
        println(n)
