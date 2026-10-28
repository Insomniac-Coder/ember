#$ test: compile-fail
#$ rules: BRW-14, BRW-15
#$ help: to change it through `p`, write `if cam.target exists as ref mut p:`
## design rules: R13, R5
## R13, R5: plain `p` after `exists as` is a read-only view; a change through it is refused (E3023)
## with the word that allows it.

class Player:
    health: int = 100

class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = weakref player
    if cam.target exists as p:
        p.health -= 1          #$ error[E3023]: `p.health -= 1` changes the player `cam.target` links to, but `p` only reads it
    println(player.health)
