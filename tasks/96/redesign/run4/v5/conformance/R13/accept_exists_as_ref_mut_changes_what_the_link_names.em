#$ test: run-pass
#$ rules: GRM-44, FN-4
#$ stdout: 99
## design rules: R13, R5
## R13, R5: the name after `exists as` is declared with its words, as a pattern name is:
## `exists as ref mut p` may change what the link points at; no first line names the change (R5,
## verdict 86).

class Player:
    health: int = 100

class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = weakref player
    if cam.target exists as ref mut p:
        p.health -= 1
    println(player.health)
