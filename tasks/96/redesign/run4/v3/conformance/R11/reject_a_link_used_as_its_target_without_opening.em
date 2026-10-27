#$ test: compile-fail
#$ rules: R11, R13
## R11: reading a link "is written as an opening (R13)"; a link used as its target without one is
## refused (AN-65, still refused).

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = weakref player
    println(cam.target.hp)              #$ error[E2020]: `cam.target` may be gone; open it: `if cam.target exists as p:`
