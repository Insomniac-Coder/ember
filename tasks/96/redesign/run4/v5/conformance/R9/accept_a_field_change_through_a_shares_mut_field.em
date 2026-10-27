#$ test: compile-pass
#$ rules: RC-1, FN-4
## design rules: R9, R5
## R9: an owner made with `shares mut`, here a field, may change the shared player; `shake` changes it
## through the field, which the compiler works out from its body (R5, verdict 86; AN-48 (a)).

class Player:
    hp: int = 100

class Camera:
    target: shares mut Player

    fn shake():
        this.target.hp -= 1

fn main():
    p = Player()
    cam = Camera(target=shares mut p)
    cam.shake()
    println(p.hp)
