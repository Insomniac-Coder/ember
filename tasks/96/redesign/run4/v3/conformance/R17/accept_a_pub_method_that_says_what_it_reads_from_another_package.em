#$ test: run-pass
#$ rules: R17, R5, R8
#$ stdout: [0] orc died
## R17: `ledger`'s `pub fn note` writes `reads frame`, so callers in this package know it reads
## neither `players` nor the player `p` names, and the call inside the loop is accepted (AN-17).

from ledger import Game, Player

fn tick(ref mut g: Game):
    for ref mut p in g.players:
        p.hp -= 1
        if p.hp <= 0:
            g.note(f"{p.name} died")

fn main():
    g = Game(players=[Player(name=String("orc"), hp=1)], log=[])
    tick(ref mut g)
    println(g.log[0])
