# Clause BRW-1.3: "unless the compiler proves the two differ (`if t is not a:`), or proves them the
# same whole variable given to one call (`[BRW-11]`)"
#$ test: run-pass
#$ rules: BRW-1, BRW-11, BCK-3, GRM-44, MONO-10
#$ stdout: 8 9
#$ stdout: 2
# Inside `if t is not p:` the link's player is not `p`. `add_both` is `[BRW-11]`'s own example: one
# whole variable under two names, compiled against a copy of the function in which they are one.

class Player:
    hp: int = 10
    target: weakref Player = None

fn hit_the_target(ref mut p: Player):
    if p.target exists as ref mut t:
        if t is not p:
            t.hp -= 1
    p.hp -= 2

fn add_both(ref mut a: int, ref mut b: int):
    a += 1
    b += 1

fn main():
    a = Player()
    b = Player()
    a.target = weakref b
    hit_the_target(ref mut a)
    println(a.hp, b.hp)
    x = 0
    add_both(ref mut x, ref mut x)
    println(x)
