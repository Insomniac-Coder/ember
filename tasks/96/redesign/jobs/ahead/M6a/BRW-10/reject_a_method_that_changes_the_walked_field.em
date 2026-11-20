# Clause BRW-10.2: "`fn note(owns msg: String):` whose body is `this.log.push(owns msg)` changes
# `this.log`, so `this.note(...)` inside a loop over `this.log` is refused (`E3020`)"
#$ test: compile-fail
#$ rules: BRW-10, BRW-18, EXC-15
# The spec's case through `this`, and the same through the object's own name. The spec gives no text
# for it: code only.

class Game:
    log: Array[String] = []

    fn note(owns msg: String):
        this.log.push(owns msg)

    fn replay():
        for line in this.log:
            this.note(owns String(line))    #$ error[E3020]

fn main():
    g = Game()
    g.note(owns String("start"))
    for line in g.log:
        g.note(owns String(line))    #$ error[E3020]
    g.replay()
