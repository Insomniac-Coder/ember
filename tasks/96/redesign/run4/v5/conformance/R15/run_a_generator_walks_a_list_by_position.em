#$ test: run-pass
#$ rules: THR-1, LT-1
#$ stdout: a
#$ stdout: b
#$ stdout: 4
#$ stdout: 5
## design rules: R15, R6
## R15: "A generator's `for` over a list walks by position across `yield`, so it holds no address into
## its own frame"; the generator protocol runs per `yield` (section 2; AN-77's NEW).

gen fn lines() -> Generator[String]:
    buf = [String("a"), String("b")]
    for s in buf:
        yield copy of s

class Door:
    angle: float = 0.0
    log: Array[String] = []

    gen fn replay() -> Generator[int] from log:
        for entry in this.log:
            yield entry.len()

fn main():
    for s in lines():
        println(s)
    d = Door(log=[String("open"), String("shut!")])
    for n in d.replay():
        println(n)
