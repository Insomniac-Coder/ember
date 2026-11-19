#$ test: run-pass
#$ rules: FN-9, FN-1, OWN-9
#$ stdout: 1
#$ stdout: 2
#$ stdout: drop 2
#$ stdout: 99
#$ stdout: drop 99
#$ stdout: 99
# `[FN-9]` — a parameter of a class type takes its word as one of any type does: `p: Player` reads
# the object, `ref mut p: Player` may change it, and `p = Player()` gives the caller's place another
# object (the old one ends), and `owns p: Player` hands the object over, so the caller's name ends.

class Player:
    hp: int

    fn heal():
        this.hp += 1

    fn drop():
        println("drop", this.hp)

fn read(p: Player) -> int:
    return p.hp

fn change(ref mut p: Player):
    p.heal()

fn replace(ref mut p: Player):
    p = Player(99)

fn take(owns p: Player) -> int:
    return p.hp

fn main():
    a = Player(1)
    println(read(a))
    change(ref mut a)
    println(a.hp)
    replace(ref mut a)
    println(a.hp)
    println(take(owns a))
