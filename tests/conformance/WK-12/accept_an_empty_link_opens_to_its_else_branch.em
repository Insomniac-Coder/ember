#$ test: run-pass
#$ rules: WK-12, WK-11
#$ stdout: gone
#$ stdout: 100
#$ stdout: gone
## Design test `run_an_empty_link_opens_to_its_else_branch.em` of `tasks/96/redesign/run4/v3/conformance/`; where it says so, `owns` is written at an `owns` parameter (`[FN-2a]`) and a `changes` list on a method's first line is left out.
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
