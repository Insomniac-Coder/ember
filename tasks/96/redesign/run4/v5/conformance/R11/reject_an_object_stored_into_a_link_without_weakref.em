#$ test: compile-fail
#$ rules: TYP-4, CLO-3
## design rules: R11, R1
## R11: a link is made with `weakref x`; `=` alone with a whole name hands the player over (R1), which a
## `weakref` place cannot take (AN-65, still refused).

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = player                 #$ error[E2020]: `=` alone would hand `player` over; write `cam.target = weakref player`
