#$ test: compile-fail
#$ rules: LT-45, TYP-15
#$ help: target: shares mut Player
## design rules: R7
## R7: a `sharedref mut` field is a view field, and a class object is never a view, so in a class it
## needs a usable `from` (E2118; the vocabulary's `sharedref` row). The message is this file's, in R18's
## form; the rules give none for this case.

class Player:
    hp: int = 100

class Camera:
    target: sharedref mut Player    #$ error[E2118]: a `Camera` keeps `target` after the player it points at may be gone; to keep the player alive write `target: shares mut Player` and make the camera with `Camera(target=shares mut p)`

fn main():
    p = Player()
    cam = Camera(target=sharedref mut p)
    println(cam.target.hp)
