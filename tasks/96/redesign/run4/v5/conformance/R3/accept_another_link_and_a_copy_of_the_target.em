#$ test: run-pass
#$ rules: R3, R11, R13
#$ stdout: 100 5
## R3, R11, R13: the nearest programs that are allowed: `weakref cam.target` makes another link to
## the same player; a copy of the player itself is made from the opened name.

@derive(Clone)
class Player:
    health: int = 100

class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = weakref player
    a = weakref cam.target
    if a exists as p:
        c = copy of p
        c.health = 5
        println(p.health, c.health)
