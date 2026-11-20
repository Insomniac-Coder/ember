# Clause BRW-21.7: "an argument for it that points into a place the callee reads or changes by
# another name while the parameter is used is refused too"
#$ test: run-pass
#$ rules: BRW-21, BRW-22, BCK-8
#$ stdout: 10
#$ stdout: 10
#$ stdout: 10
# Only while the parameter is used: `heal_then_show` reads `PARTY[0]` after `p`'s last use, and
# `heal` is given a player that is not in `PARTY`.

class Player:
    hp: int = 0

static mut PARTY: Array[Player] = []

fn heal(ref mut p: Player):
    println(PARTY[0].hp)
    p.hp += 10

fn heal_then_show(ref mut p: Player):
    p.hp += 10
    println(PARTY[0].hp)

fn main():
    PARTY.push(owns Player())
    heal_then_show(ref mut PARTY[0])
    mine = Player()
    heal(ref mut mine)
    println(mine.hp)
