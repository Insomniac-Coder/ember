# Clause FN-9.3: "`owns p: Player` hands the object over, so the caller's name ends"
#$ test: compile-fail
#$ rules: FN-9, OWN-3, CLS-1
# No text in the spec for this program: code only.

class Player:
    hp: int = 0

static mut KEPT: Array[Player] = []

fn keep(owns p: Player):
    KEPT.push(owns p)

fn main():
    a = Player()
    keep(owns a)
    println(a.hp)    #$ error[E3040]
