#$ test: compile-fail
#$ rules: R5
#$ help: fn tick() changes players, log:
## R5: a function that calls one whose first line changes something it can reach names that too:
## `tick` calls `note`, which changes `log` (AN-17, E2105).

class Player:
    name: String
    hp: int

class Game:
    players: Array[Player]
    log: Array[String]
    frame: int = 0

    fn note(msg: String) changes log:
        this.log.push(f"[{this.frame}] {msg}")

    fn tick() changes players:
        for ref mut p in this.players:
            p.hp -= 1
            if p.hp <= 0:
                this.note(f"{p.name} died")    #$ error[E2105]: `tick` calls `note`, which changes `log`; add it to `tick`'s first line: `fn tick() changes players, log:`

fn main():
    g = Game(players=[Player(name="orc", hp=1)], log=[])
    g.tick()
    println(g.log[0])
