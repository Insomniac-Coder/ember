# Clause EXC-2.1: "A read of an object's field through another name while a `ref mut` name of that
# field is still used is `E3021` (`[BRW-1]`)."
#$ test: compile-fail
#$ rules: EXC-2, BRW-1
# The other name is a link, which may lead to `p`'s object. No text in the spec: code only.

class Player:
    hp: int = 5

class Camera:
    target: weakref Player = None

fn main():
    p = Player()
    cam = Camera()
    cam.target = weakref p
    r = ref mut p.hp
    if cam.target exists as t:
        println(t.hp)    #$ error[E3021]
    r += 1
