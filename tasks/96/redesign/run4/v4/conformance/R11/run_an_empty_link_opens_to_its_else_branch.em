#$ test: run-pass
#$ rules: R11, R13
#$ stdout: gone
#$ stdout: 100
#$ stdout: gone
## R11: a link "may hold no target (`... = None`)"; "Reading one tests the target's header (does the
## object still live?) and is written as an opening (R13)": an empty link and a link whose player ended
## both open to `else:`. Section 2: "one test of the target's header, each read of a `weakref` link".

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None

fn show(cam: Camera):
    if cam.target exists as p:
        println(p.hp)
    else:
        println("gone")

fn main():
    cam = Camera()
    show(cam)                           # gone: the link holds no target
    player = Player()
    cam.target = weakref player
    show(cam)                           # 100
    player = Player(hp=5)               # the player the link names ends
    show(cam)                           # gone
