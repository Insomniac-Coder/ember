# Clause BRW-1.4: "For a `ref mut` parameter and a place its function reaches by its own name (a
# module variable), the check is made at each call, by what the body does (`[BRW-21]`), so the
# function itself compiles."
#$ test: run-pass
#$ rules: BRW-1, BRW-21, LT-3, STA-1
#$ stdout: 10
#$ stdout: 20
# `heal` reads `PARTY[0]` by its own name while `p` is used: it compiles, and a call giving it a
# player that is not `PARTY[0]` is accepted. `heal(ref mut PARTY[0])` is refused at the call, which
# is `[BRW-21]`'s test.

struct Player:
    hp: int

static mut PARTY: Array[Player] = [Player(hp=10)]

fn heal(ref mut p: Player):
    println(PARTY[0].hp)
    p.hp += 10

fn main():
    spare = Player(hp=10)
    heal(ref mut spare)
    println(spare.hp)
