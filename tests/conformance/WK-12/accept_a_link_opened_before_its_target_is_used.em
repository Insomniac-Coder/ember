#$ test: compile-pass
#$ rules: WK-12, GRM-44
## Design test `accept_a_link_opened_before_its_target_is_used.em` of `tasks/96/redesign/run4/v3/conformance/`; where it says so, `owns` is written at an `owns` parameter (`[FN-2a]`) and a `changes` list on a method's first line is left out.
## R11: the nearest allowed program: the link is opened, and the opened name is used (AN-65's NEW).

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = weakref player
    if cam.target exists as p:
        println(p.hp)
