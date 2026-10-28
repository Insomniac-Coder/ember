#$ test: compile-fail
#$ rules: R5
#$ help: fn note(msg: String) reads frame changes log:
## R5: the parts of a first line go in one order, parameters, `reads`, `changes`, `-> T`, `from`, so
## every first line reads the same way (E0118).

class Game:
    log: Array[String]
    frame: int = 0

    fn note(msg: String) changes log reads frame:    #$ error[E0118]: the parts of `note`'s first line are out of order; write `fn note(msg: String) reads frame changes log:`
        this.log.push(f"[{this.frame}] {msg}")

fn main():
    g = Game(log=[])
    g.note(String("start"))
    println(g.log[0])
