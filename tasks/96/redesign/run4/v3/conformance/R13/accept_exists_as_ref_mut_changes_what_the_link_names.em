#$ test: run-pass
#$ rules: R13, R5
#$ stdout: 99
## R13, R5: the name after `exists as` is declared with its words, as a pattern name is:
## `exists as ref mut p` may change what the link points at; `main` changes a `Player` through a link,
## so its first line names it (R5).

class Player:
    health: int = 100

class Camera:
    target: weakref Player = None

fn main() changes Player:
    player = Player()
    cam = Camera()
    cam.target = weakref player
    if cam.target exists as ref mut p:
        p.health -= 1
    println(player.health)
