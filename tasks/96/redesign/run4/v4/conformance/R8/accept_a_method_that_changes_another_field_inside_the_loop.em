#$ test: run-pass
#$ rules: R8, R5
#$ stdout: ['[0] orc died']
## R8 (c), (f), row 3: `note` changes only `log` by its first line, so it may run while the loop walks
## `players` and while `p` is the only name for one player. AN-17's NEW program.

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
    println(g.log)
