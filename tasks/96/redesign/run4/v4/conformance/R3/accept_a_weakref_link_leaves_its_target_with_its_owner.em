#$ test: run-pass
#$ rules: R3, R11, R13
#$ stdout: 100
#$ 100
## R3, R11, R13: the nearest program that is allowed: `weakref player` makes a link that does not keep
## the player alive, and `player` stays `main`'s own (AN-65's NEW).

class Player:
    health: int = 100

class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = weakref player
    if cam.target exists as p:
        println(p.health)
    println(player.health)
