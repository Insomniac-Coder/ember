#$ test: compile-pass
#$ rules: OWN-8
## design rules: R11, R3
## R11: the nearest allowed program to `reject_copy_of_a_link.em`: `weakref` before a link makes
## another link to the same target, with its own weak count (verdict 10).

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = weakref player
    a = weakref cam.target
    if a exists as p:
        println(p.hp)
