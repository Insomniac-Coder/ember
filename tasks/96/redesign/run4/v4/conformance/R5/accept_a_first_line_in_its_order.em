#$ test: run-pass
#$ rules: R5
#$ stdout: [0] start
## R5: the nearest allowed program: `reads frame` before `changes log`, R5's order.

class Game:
    log: Array[String]
    frame: int = 0

    fn note(msg: String) reads frame changes log:
        this.log.push(f"[{this.frame}] {msg}")

fn main():
    g = Game(log=[])
    g.note(String("start"))
    println(g.log[0])
