#$ test: compile-fail
#$ rules: R3, R11
#$ help: write `cam.target = weakref player`
## R3, R11: a `weakref T` place takes a link made with `weakref`; `=` alone would hand the player over
## (E2020; the `Weak` row of `V3/vocabulary.md`, AN-65's message).

class Player:
    health: int = 100

class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = player        #$ error[E2020]: `=` alone would hand `player` over
    println(player.health)
