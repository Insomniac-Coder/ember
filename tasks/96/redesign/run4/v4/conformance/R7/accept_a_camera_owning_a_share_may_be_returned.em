#$ test: run-pass
#$ rules: R7, R9, R5
#$ stdout: 99
## R7: the way out E3063's message gives: a camera that owns a share of the player (`shares mut`, R9) is
## not a view, so it may outlive the function that made the player. AN-48's form (a). `main` changes
## the player through the camera's link (`cam.shake()`), so it says `changes Player` (R5, verdicts 63, 64).

class Player:
    hp: int = 100

struct Camera:
    target: shares mut Player

    fn shake() changes target:
        this.target.hp -= 1

fn make_camera() -> Camera:
    p = Player()
    cam = Camera(target=shares mut p)
    return cam

fn main() changes Player:
    cam = make_camera()
    cam.shake()
    println(cam.target.hp)
