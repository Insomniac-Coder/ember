# Clause BRW-12.9: "the link words make a link (`[OWN-13]`)"
#$ test: run-pass
#$ rules: BRW-12, OWN-13, WK-11, GRM-44
#$ stdout: 3
#$ stdout: gone
# `weakref players[1]` is a link to the object, which does not keep it alive.

class Player:
    hp: int = 10

fn main():
    players = [Player(), Player(hp=3)]
    w = weakref players[1]
    if w exists as p:
        println(p.hp)
    _ = players.pop()
    if w exists as p:
        println(p.hp)
    else:
        println("gone")
