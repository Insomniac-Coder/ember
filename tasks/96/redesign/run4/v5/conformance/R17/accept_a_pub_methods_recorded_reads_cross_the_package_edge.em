#$ test: run-pass
#$ rules: BLD-14, FN-4, BRW-16
#$ stdout: [0] orc died
## design rules: R17, R5, R8
## R17: `ledger`'s interface file records that `pub fn note` reads only `frame` and `log` (verdict 86:
## the fact is worked out from its body, with no `reads` word), so callers in this package know it reads
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
