#$ test: compile-fail
#$ rules: TYP-4, CLO-3
#$ help: open it: `if cam.target exists as p:`
## design rules: R13, R11
## R13, R11: a `weakref` link's target may be gone, so it is read only by opening it; using the link
## as its target is refused (E2020; AN-65's message).

class Player:
    health: int = 100

class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = weakref player
    println(cam.target.health) #$ error[E2020]: `cam.target` may be gone
