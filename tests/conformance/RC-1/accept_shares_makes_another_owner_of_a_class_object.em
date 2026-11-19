#$ test: run-pass
#$ rules: RC-1, OWN-13
#$ profiles: debug, development, release
#$ stdout: 10
#$ 7
#$ 14
# `[RC-1]` — `shares x` makes one more owner of what `x` names, on this thread, that reads it; an
# owner reads as its object (its fields and methods are the object's), and another owner may be
# made from an owner.

class Player:
    hp: int

    fn hit(n: int) -> int:
        return this.hp - n

fn main():
    p = Player(hp=10)
    s: shares Player = shares p
    println(s.hp)
    println(s.hit(3))
    q = Player(hp=7)
    a = shares q
    b = shares a
    println(b.hp + a.hp)
