# Clause EXC-2.1: "while a `ref mut` name of that field is still used"
#$ test: run-pass
#$ rules: EXC-2, BRW-1, BRW-2
#$ stdout: 6
#$ stdout: 6
# The read goes through `r` while `r` is used, and through the link after `r`'s last use.

class Player:
    hp: int = 5

class Camera:
    target: weakref Player = None

fn main():
    p = Player()
    cam = Camera()
    cam.target = weakref p
    r = ref mut p.hp
    r += 1
    println(r)
    if cam.target exists as t:
        println(t.hp)
