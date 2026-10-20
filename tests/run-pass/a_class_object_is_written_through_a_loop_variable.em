#$ test: run-pass
#$ rules: FN-9
#$ stdout: 100.0 100.0

## D-747 — `for p in players` over a borrowed `Array[Player]` gives `ref Player`, and writing
## `p.health` was `E3021`, "cannot write through a shared reference": the handle is only read, and a
## class object's fields are written through any handle (`[FN-9]`; VII.7's own `heal_all`).

class Player:
    name: String
    health: float = 10.0

fn heal_all(players: Array[Player]):
    for p in players:
        p.health = 100.0

fn main():
    ps: Array[Player] = [Player("a"), Player("b")]
    heal_all(ps)
    println(ps[0].health, ps[1].health)
