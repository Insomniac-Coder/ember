#$ test: run-pass
#$ rules: RC-5, WK-11, GRM-44
#$ stdout: 100
## design rules: R10, R11, R13
## R10 point 1: "No line or call in the view's life may end an object of that type", and "nothing runs
## for it while the program runs": nothing in `p`'s life can end a `Player`, so `p` costs no count; the
## opening is one test of the player's header (AN-65; section 2's rows "nothing" and "one test of the
## target's header"). Checked in the emitted C: no count change between the opening and `p`'s last use.

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
