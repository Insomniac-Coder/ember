# Clause BRW-10.2: "and inside a loop over `this.players` is allowed"
#$ test: run-pass
#$ rules: BRW-10, BRW-18, EXC-19
#$ stdout: 2
#$ stdout: ann
#$ stdout: bob
# `note` changes only `this.log`, so it may be called while a loop walks `this.players`.

class Game:
    log: Array[String] = []
    players: Array[String] = []

    fn note(owns msg: String):
        this.log.push(owns msg)

    fn greet_all():
        for p in this.players:
            this.note(owns String(p))

fn main():
    g = Game()
    g.players.push(owns String("ann"))
    g.players.push(owns String("bob"))
    g.greet_all()
    println(g.log.len())
    for line in g.log:
        println(line)
