#$ test: compile-fail
#$ rules: WK-16, WK-12, TYP-45
# `[WK-16]` — a `weakref` link used as its target without opening it is `E2020`: the target may be
# gone, and the link is read by opening it.

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = weakref player
    println(cam.target.hp)            #$ error[E2020]: `cam.target` may be gone; open it: `if cam.target exists as p:`
