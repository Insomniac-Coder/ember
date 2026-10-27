#$ test: compile-fail
#$ rules: TYP-4, CLO-3
## design rules: R11, R13
## R11: reading a link "is written as an opening (R13)"; a link used as its target without one is
## refused (AN-65, still refused). The message gives the lines that replace line 16.

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = weakref player
    println(cam.target.hp)              #$ error[E2020]: `cam.target` may be gone; open it: write `if cam.target exists as p:` in place of line 16, and `println(p.hp)` indented under it
