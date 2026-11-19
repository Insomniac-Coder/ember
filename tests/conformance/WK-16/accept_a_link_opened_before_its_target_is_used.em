#$ test: run-pass
#$ rules: WK-16, WK-12, WK-11
#$ profiles: debug, development, release
#$ stdout: 100
#$ 100
# `[WK-16]` — a link is opened before its target is used, and `weakref x` stores an object into a
# link place; `weakref` before a link makes another link to the same target.

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = weakref player
    if cam.target exists as p:
        println(p.hp)
    a = weakref cam.target
    if a exists as q:
        println(q.hp)
