#$ test: run-pass
#$ rules: OWN-1, OWN-8
#$ stdout: 3
#$ ann
## design rules: R1, R3
## R1, R3: the nearest program that is allowed for both refusals above: each round pushes a player
## of its own, and the caller keeps its player (AN-35's NEW `spawn_all`).

@derive(Clone)
class Player:
    name: String

fn spawn_all(template: Player, n: int) -> Array[Player]:
    v: Array[Player] = []
    for _ in 0..n:
        v.push(copy of template)
    return v

fn main():
    p = Player(name=String("ann"))
    println(spawn_all(p, 3).len())
    println(p.name)
