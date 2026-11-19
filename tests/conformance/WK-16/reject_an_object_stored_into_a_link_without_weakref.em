#$ test: compile-fail
#$ rules: WK-16, TYP-45, OWN-3
# `[WK-16]` — an object stored into a `weakref` place without `weakref` is `E2020`: `=` alone would
# hand the object over, which a link cannot take.

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = player               #$ error[E2020]: `=` alone would hand `player` over; write `cam.target = weakref player`
