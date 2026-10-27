#$ test: compile-fail
#$ rules: GRM-44
#$ help: `exists as` declares a new name: write `if cam.target exists as t:`
## design rules: R13
## R13: the name after `exists as` is new: a name already in use is refused (E2106; `V4/rules.md`
## section 3's message).

class Player:
    health: int = 100

class Camera:
    target: weakref Player = None

fn main():
    target = 1
    player = Player()
    cam = Camera()
    cam.target = weakref player
    if cam.target exists as target:    #$ error[E2106]: `target` already names a value (line 14)
        println("still here")
    println(target)
