#$ test: run-pass
#$ rules: R5, R8
#$ stdout: start
#$ stdout: [0] replayed
## R5, R8 (c), row 3: the nearest program that is allowed: `this.note(...)`, which pushes to `this.log`
## (worked out from its body, verdict 86), runs after the loop over `this.log`.

class Game:
    log: Array[String]
    frame: int = 0

    fn note(msg: String):
        this.log.push(f"[{this.frame}] {msg}")

    fn replay():
        for line in this.log:
            println(line)
        this.note(String("replayed"))

fn main():
    g = Game(log=[String("start")])
    g.replay()
    println(g.log[1])
