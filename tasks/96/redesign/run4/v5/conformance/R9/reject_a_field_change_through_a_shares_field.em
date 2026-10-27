#$ test: compile-fail
#$ rules: R9, R5
## R9: a field written `shares T` is an owner that only reads its value; the camera changes the
## player only through a `shares mut` field (AN-48, still refused). The two lines the message gives make
## the program compile (`accept_a_field_change_through_a_shares_mut_field.em`).

class Player:
    hp: int = 100

class Camera:
    target: shares Player

    fn shake():
        this.target.hp -= 1       #$ error[E3023]: `this.target` only reads the player it shares; to change it through the camera write `target: shares mut Player` in place of line 11 and `cam = Camera(target=shares mut p)` in place of line 18

fn main():
    p = Player()
    cam = Camera(target=shares p)
    cam.shake()
