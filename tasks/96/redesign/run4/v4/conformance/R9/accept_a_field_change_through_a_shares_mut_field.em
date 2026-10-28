#$ test: compile-pass
#$ rules: R9, R5
## R9: an owner made with `shares mut`, here a field, may change the shared player; `shake`'s first
## line names the field it changes through ("anything reached through them", R5; AN-48 (a)). `main`
## changes the player through its camera's link, so it says `changes Player` (R5, verdict 64).

class Player:
    hp: int = 100

class Camera:
    target: shares mut Player

    fn shake() changes target:
        this.target.hp -= 1

fn main() changes Player:
    p = Player()
    cam = Camera(target=shares mut p)
    cam.shake()
    println(p.hp)
