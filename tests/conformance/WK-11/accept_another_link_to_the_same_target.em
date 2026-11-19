#$ test: compile-pass
#$ rules: WK-11, WK-12
## Design test `accept_another_link_to_the_same_target.em` of `tasks/96/redesign/run4/v3/conformance/`; where it says so, `owns` is written at an `owns` parameter (`[FN-2a]`) and a `changes` list on a method's first line is left out.
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
