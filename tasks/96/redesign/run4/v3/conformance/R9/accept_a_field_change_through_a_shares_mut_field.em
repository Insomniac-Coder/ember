#$ test: compile-pass
#$ rules: R9, R5
## R9: an owner made with `shares mut`, here a field, may change the shared player; `shake`'s first
## line names the field it changes through (AN-48 (a)).

class Player:
    hp: int = 100

class Camera:
    target: shares mut Player

    fn shake() changes target:
        this.target.hp -= 1

fn main():
    p = Player()
    cam = Camera(target=shares mut p)
    cam.shake()
    println(p.hp)
