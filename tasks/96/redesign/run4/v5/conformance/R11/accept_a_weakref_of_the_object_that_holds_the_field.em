#$ test: run-pass
#$ rules: WK-16
#$ stdout: 3
## design rules: R11, R13
## R11: the nearest allowed program to `reject_a_weakref_of_a_struct_field_of_an_object.em`, with the
## lines its message gives: the link names the player, which lives in its own block, and the field is
## read through the opened name.

struct Pos:
    x: int = 0

class Player:
    pos: Pos = Pos(x=3)

class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = weakref player
    if cam.target exists as p:
        println(p.pos.x)
