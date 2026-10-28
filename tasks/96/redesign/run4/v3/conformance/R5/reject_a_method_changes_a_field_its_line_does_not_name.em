#$ test: compile-fail
#$ rules: R5
#$ help: fn note(msg: String) changes log, frame:
## R5, row 3: `changes log` lets `note` change exactly `log`; a body that changes another field does
## not match its first line (AN-17, E2105).

class Game:
    log: Array[String]
    frame: int = 0

    fn note(msg: String) changes log:
        this.log.push(copy of msg)
        this.frame += 1    #$ error[E2105]: `note` changes `this.frame`, which its first line does not name; write `fn note(msg: String) changes log, frame:`

fn main():
    g = Game(log=[])
    g.note(String("start"))
    println(g.log[0])
