#$ test: run-pass
#$ rules: R5, R8, R17
#$ stdout: [0] orc died
## R5, row 3: `this.note(...)` runs while `p` is the only name of one player (R8 (f)), accepted because
## `note`'s body changes only `log` and reads only `frame` and `log`, worked out with no word on any
## first line (verdict 86; AN-17's NEW). Inside a loop over `this.log` the same call is refused
## (`reject_a_method_that_adds_to_the_log_called_in_a_loop_over_the_log`).

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
    println(g.log[0])
