#$ test: compile-fail
#$ rules: R11, R13
## R11, R13: a link is opened with `exists as`; today's `upgrade` is gone (AN-65, still refused). The
## message gives every line it replaces, so following it compiles.

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = weakref player
    p = cam.target.upgrade()            #$ error[E1010]: `upgrade` is not Ember: a `weakref` link is opened with `exists as`; write `if cam.target exists as p:` in place of line 16, and indent line 17 under it
    println(p.hp)
