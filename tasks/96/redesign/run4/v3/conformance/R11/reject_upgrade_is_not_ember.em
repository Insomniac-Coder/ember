#$ test: compile-fail
#$ rules: R11, R13
## R11, R13: a link is opened with `exists as`; today's `upgrade` is gone (AN-65, still refused).

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = weakref player
    p = cam.target.upgrade()            #$ error[E1010]: a `weakref` link is opened with `exists as`: `if cam.target exists as p:`
    println(p)
