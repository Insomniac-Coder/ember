#$ test: run-pass
#$ rules: R10, R11, R13
#$ stdout: 100
## R10 point 1: "Nothing, when no line in the view's life can end an owner of an object of that type":
## nothing in `p`'s life can end a `Player`, so `p` costs no count and no region; the opening is one
## test of the player's header (AN-65; section 2's R10 row, "nothing"). Checked in the emitted C: no
## count change and no region counter between the opening and `p`'s last use.

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = weakref player
    if cam.target exists as p:          # the player still lives: call it `p`
        println(p.hp)
    else:
        println("gone")
