#$ test: compile-fail
#$ rules: WK-12, TYP-45, GRM-44
## Design test `reject_upgrade_is_not_ember.em` of `tasks/96/redesign/run4/v3/conformance/`; where it says so, `owns` is written at an `owns` parameter (`[FN-2a]`) and a `changes` list on a method's first line is left out.
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
