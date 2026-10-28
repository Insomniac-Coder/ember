#$ test: run-pass
#$ rules: BLD-14, FN-4, BRW-16
#$ stdout: [0] orc died
## design rules: R17, R5, R8
## R17: inside a package, facts worked out from a body are used freely: `note_any`'s body, seen here,
## reads only `frame` and `log`, so it may run while `p` is the only name of one player (R8 (f)); `pub`
## changes nothing inside the package (AN-17). Across the edge the same fact comes from the interface
## file (`accept_a_pub_methods_recorded_reads_cross_the_package_edge`).

class Player:
    name: String
    hp: int

class Game:
    players: Array[Player]
    log: Array[String]
    frame: int = 0

    pub fn note_any(msg: String):
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
