#$ test: compile-pass
#$ rules: WK-16, WK-11
## Design test `accept_an_object_stored_into_a_link_with_weakref.em` of `tasks/96/redesign/run4/v3/conformance/`; where it says so, `owns` is written at an `owns` parameter (`[FN-2a]`) and a `changes` list on a method's first line is left out.
## R11: the nearest allowed program: `weakref player` makes the link, and `player` stays `main`'s.

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = weakref player
    println(player.hp)
