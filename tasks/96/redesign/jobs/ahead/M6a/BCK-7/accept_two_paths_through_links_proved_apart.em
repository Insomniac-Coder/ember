# Clause BCK-7.1: "two paths to such places follow `[BCK-3]`'s rule for links" (`[BCK-3]`: "unless
# the compiler knows they differ (different types, a fresh object, under `if a is not b:`)")
#$ test: run-pass
#$ rules: BCK-7, BCK-3, EXC-3
#$ stdout: 6
# The same two links, proved apart by `if t is not u:`.

class Player:
    hp: int = 5

class Camera:
    target: weakref Player = None

fn main():
    players = [Player(), Player()]
    a = Camera()
    b = Camera()
    a.target = weakref players[0]
    b.target = weakref players[1]
    if a.target exists as ref mut t:
        if b.target exists as u:
            if t is not u:
                t.hp = u.hp + 1
                println(t.hp)
