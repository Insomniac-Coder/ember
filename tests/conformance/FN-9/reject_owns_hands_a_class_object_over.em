#$ test: compile-fail
#$ rules: FN-9, FN-1, OWN-3
# `[FN-9]` — `owns p: Player` hands the object over, so the caller's name ends: a later use is `E3040`.

class Player:
    hp: int

fn take(owns p: Player) -> int:
    return p.hp

fn main():
    a = Player(1)
    println(take(owns a))
    println(a.hp)    #$ error[E3040]: `a` was handed over to `take` on line 13 and is used on line 14; to keep both, hand over a copy of your own: `copy of a`
