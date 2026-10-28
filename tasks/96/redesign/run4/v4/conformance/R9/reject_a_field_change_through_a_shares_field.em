#$ test: compile-fail
#$ rules: R9, R5
## R9: a field written `shares T` is an owner that only reads its value; the camera changes the
## player only through a `shares mut` field (AN-48, still refused). `main` already says `changes Player`
## (it calls `shake`, whose first line changes what `target` reaches, a player behind a link: R5,
## verdict 64), so the two lines the message gives make the program compile.

class Player:
    hp: int = 100

class Camera:
    target: shares Player

    fn shake() changes target:
        this.target.hp -= 1       #$ error[E3023]: `this.target` only reads the player it shares; to change it through the camera write `target: shares mut Player` in place of line 12 and `cam = Camera(target=shares mut p)` in place of line 19

fn main() changes Player:
    p = Player()
    cam = Camera(target=shares p)
    cam.shake()
