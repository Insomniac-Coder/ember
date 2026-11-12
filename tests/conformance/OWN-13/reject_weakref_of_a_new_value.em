#$ test: compile-fail
#$ rules: OWN-13, WK-16
# `[OWN-13]` — `weakref` before a new value is `E2119`: the value would end at the end of the line,
# and the link would find nothing.

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None

fn main():
    cam = Camera()
    cam.target = weakref Player()     #$ error[E2119]: the player would end at the end of the line and the link would find nothing; keep it in a name first: write `p = Player()` above, then `cam.target = weakref p`
