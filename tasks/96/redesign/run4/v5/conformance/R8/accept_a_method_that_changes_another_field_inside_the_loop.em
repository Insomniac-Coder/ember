#$ test: run-pass
#$ rules: BRW-16, FN-4
#$ stdout: ['[0] orc died']
## design rules: R8, R5
## R8 (c), (f), row 3: `note` changes only `log`, by its body, so it may run while the loop walks
## `players` and while `p` is the only name for one player. AN-17's NEW program.

class Player:
    name: String
    hp: int

class Game:
    players: Array[Player]
    log: Array[String]
    frame: int = 0

    fn note(msg: String):
        this.log.push(f"[{this.frame}] {msg}")

    fn tick():
        for ref mut p in this.players:
            p.hp -= 1
            if p.hp <= 0:
                this.note(f"{p.name} died")

fn main():
    g = Game(players=[Player(name="orc", hp=1)], log=[])
    g.tick()
    println(g.log)
