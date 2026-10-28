#$ test: compile-fail
#$ rules: R9, R5
## R9: a field written `shares T` is an owner that only reads its value; the camera changes the
## player only through a `shares mut` field (AN-48, still refused).

class Player:
    hp: int = 100

class Camera:
    target: shares Player

    fn shake() changes target:
        this.target.hp -= 1       #$ error[E3023]: `this.target` only reads the player it shares; to change it through the camera write `target: shares mut Player` and `Camera(target=shares mut p)`

fn main():
    p = Player()
    cam = Camera(target=shares p)
    cam.shake()
