#$ test: run-pass
#$ rules: BRW-16, GRM-44, FN-4
#$ stdout:
#$ 10
#$ 20
#$ 9
## design rules: R8, R13, R5
## R8 (b), (g): inside `if q is not p:` the compiler knows the opened name and the loop name are two
## players, so the change through the link is allowed while `p` reads. Nothing in the loop can end a
## player, so nothing runs for `q` (R10 point 1). Completeness check, G25.

class Player:
    hp: int = 10

fn main():
    players: Array[Player] = []
    first = Player(hp=10)
    w = weakref first
    players.push(first)
    players.push(Player(hp=20))
    for p in players:
        if w exists as ref mut q:
            if q is not p:
                q.hp -= 1
        println(p.hp)
    if w exists as r:
        println(r.hp)
