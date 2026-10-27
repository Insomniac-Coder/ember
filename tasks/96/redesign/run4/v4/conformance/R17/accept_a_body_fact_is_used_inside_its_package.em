#$ test: run-pass
#$ rules: R17, R5, R8
#$ stdout: [0] orc died
## R17: inside a package, facts worked out from a body are used freely: `note_any` says no `reads`,
## but its body, seen here, reads only `frame` and `log`, so it may run while `p` is the only name of
## one player (R8 (f)); `pub` changes nothing inside the package (AN-17).

class Player:
    name: String
    hp: int

class Game:
    players: Array[Player]
    log: Array[String]
    frame: int = 0

    pub fn note_any(msg: String) changes log:
        this.log.push(f"[{this.frame}] {msg}")

fn tick(ref mut g: Game):
    for ref mut p in g.players:
        p.hp -= 1
        if p.hp <= 0:
            g.note_any(f"{p.name} died")

fn main():
    g = Game(players=[Player(name=String("orc"), hp=1)], log=[])
    tick(ref mut g)
    println(g.log[0])
