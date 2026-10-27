#$ test: run-pass
#$ rules: R8, R6
#$ stdout:
#$ 4
#$ 4
#$ 6
## R8 (c): the list changes after the stored generator's last use, as the refusal's message says; a
## second walk sees the new entry.

class Door:
    log: Array[String] = []

    gen fn replay() -> Generator[int] from log:
        for entry in this.log:
            yield entry.len()

fn main():
    d = Door()
    d.log.push(String("shut"))
    g = d.replay()
    println(g.next().unwrap())
    d.log.push(String("opened"))
    for n in d.replay():
        println(n)
