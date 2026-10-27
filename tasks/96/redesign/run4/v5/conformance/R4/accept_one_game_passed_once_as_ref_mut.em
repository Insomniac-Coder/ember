#$ test: run-pass
#$ rules: FN-1, FN-4
#$ stdout: orc moved
## design rules: R4, R5
## R4: the nearest allowed program to a read and a change of one game in one call: the game is passed
## once, as `ref mut g`; the loop name `p` reads `g.players` while `g.note(...)` changes only `log`
## (AN-18's NEW).

class Game:
    players: Array[String]
    log: Array[String]
    score: int = 0

    fn note(msg: String):
        this.log.push(copy of msg)

fn tick(ref mut g: Game):
    for p in g.players:
        g.note(f"{p} moved")

fn main():
    g = Game(players=[String("orc")], log=[])
    tick(ref mut g)
    println(g.log[0])
