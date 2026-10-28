#$ test: run-pass
#$ rules: GRM-44
#$ stdout: 100
#$ 1
## design rules: R13
## R13: the nearest program that is allowed: the opening declares a new name (AN-65's NEW).

class Player:
    health: int = 100

class Camera:
    target: weakref Player = None

fn main():
    target = 1
    player = Player()
    cam = Camera()
    cam.target = weakref player
    if cam.target exists as t:
        println(t.health)
    println(target)
