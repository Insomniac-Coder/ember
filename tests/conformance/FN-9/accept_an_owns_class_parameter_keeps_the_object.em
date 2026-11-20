# Clause FN-9.3: "`owns p: Player` hands the object over, so the caller's name ends"
#$ test: run-pass
#$ rules: FN-9, OWN-3
#$ stdout: 3
#$ stdout: 1
# The callee keeps the object it was handed.

class Player:
    hp: int = 0

static mut KEPT: Array[Player] = []

fn keep(owns p: Player):
    KEPT.push(owns p)

fn main():
    a = Player()
    a.hp = 3
    keep(owns a)
    println(KEPT[0].hp)
    println(KEPT.len())
