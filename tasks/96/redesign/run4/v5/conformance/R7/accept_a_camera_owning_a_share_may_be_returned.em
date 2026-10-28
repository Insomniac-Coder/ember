#$ test: run-pass
#$ rules: LT-45, RC-1, FN-4
#$ stdout: 99
## design rules: R7, R9, R5
## R7: the way out E3063's message gives: a camera that owns a share of the player (`shares mut`, R9) is
## not a view, so it may outlive the function that made the player. AN-48's form (a). No first line
## names the change `cam.shake()` makes: the compiler works it out from `shake`'s body (R5, verdict 86).

class Player:
    hp: int = 100

struct Camera:
    target: shares mut Player

    fn shake():
        this.target.hp -= 1

fn make_camera() -> Camera:
    p = Player()
    cam = Camera(target=shares mut p)
    return cam

fn main():
    cam = make_camera()
    cam.shake()
    println(cam.target.hp)
