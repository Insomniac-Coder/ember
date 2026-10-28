#$ test: run-pass
#$ rules: R5, R8, R17
#$ stdout: [0] orc died
## R5, row 3: `tick` names `players` and `log`; `this.note(...)` runs while `p` is the only name of one
## player (R8 (f)), accepted because `note`'s line changes only `log` and its body, seen inside the
## package, reads only `frame` and `log` (R17; AN-17's NEW).

class Player:
    name: String
    hp: int

class Game:
    players: Array[Player]
    log: Array[String]
    frame: int = 0

    fn note(msg: String) changes log:
        this.log.push(f"[{this.frame}] {msg}")

    fn tick() changes players, log:
        for ref mut p in this.players:
            p.hp -= 1
            if p.hp <= 0:
                this.note(f"{p.name} died")

fn main():
    g = Game(players=[Player(name="orc", hp=1)], log=[])
    g.tick()
    println(g.log[0])
