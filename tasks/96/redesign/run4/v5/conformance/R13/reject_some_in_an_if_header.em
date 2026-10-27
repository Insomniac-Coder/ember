#$ test: compile-fail
#$ rules: GRM-44
#$ help: write `if cam.target exists as p:`
## design rules: R13
## R13: `Some(...)` is written to make a value and in a `match` branch, not in an `if` header (E0115;
## the owner's verdict on run 3's `if Some(ref mut me) = ui:`; AN-65's message).

class Player:
    health: int = 100

class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = weakref player
    if Some(p) = cam.target:   #$ error[E0115]: an `Option` or a `weakref` link is opened with `exists as`
        println(p.health)
