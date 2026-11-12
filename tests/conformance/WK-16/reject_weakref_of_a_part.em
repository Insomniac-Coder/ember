#$ test: compile-fail
#$ rules: WK-16, WK-11
# `[WK-16]` — a `weakref` of a part (a field held in its object) is `E2119`: the part does not live
# on its own; the link names the object.

struct Vec2:
    x: float
    y: float

class Player:
    pos: Vec2

class Camera:
    target: weakref Vec2 = None

fn main():
    player = Player(pos=Vec2(x=1.0, y=2.0))
    cam = Camera()
    cam.target = weakref player.pos   #$ error[E2119]: a `weakref` link needs an object that lives on its own; link to the player: `cam.target = weakref player`
