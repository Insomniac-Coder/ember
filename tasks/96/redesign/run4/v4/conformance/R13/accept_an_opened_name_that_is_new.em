#$ test: run-pass
#$ rules: R13
#$ stdout: 100
#$ 1
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
