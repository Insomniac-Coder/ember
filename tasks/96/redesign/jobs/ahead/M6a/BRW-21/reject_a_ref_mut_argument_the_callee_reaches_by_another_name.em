# Clause BRW-21.7: "while it is used, the callee reaches its place through no other name, so an
# argument for it that points into a place the callee reads or changes by another name while the
# parameter is used is refused too"
#$ test: compile-fail
#$ rules: BRW-21, BRW-22, BCK-8
# The first message is the spec's, with this file's line numbers. `boost` changes `PARTY[0]` by its
# own name while `p` is still used: writing a number ends nothing, but it is a change by another
# name, which the `ref mut` parameter's promise rules out. Code only.

class Player:
    hp: int = 0

static mut PARTY: Array[Player] = []

fn heal(ref mut p: Player):
    println(PARTY[0].hp)
    p.hp += 10

fn boost(ref mut p: Player):
    PARTY[0].hp = 1
    p.hp += 10

fn main():
    PARTY.push(owns Player())
    heal(ref mut PARTY[0])    #$ error[E3031]: `heal` reads `PARTY[0]` by its own name (its line 16) while `p` is still used (its line 17), and `p` is `PARTY[0]`; in `heal`, read it through `p`: `println(p.hp)`
    boost(ref mut PARTY[0])    #$ error[E3031]
