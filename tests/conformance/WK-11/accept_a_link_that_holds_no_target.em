#$ test: run-pass
#$ rules: WK-11, WK-12, TYP-45
#$ profiles: debug, development, release
#$ stdout: gone
#$ 100
#$ gone
#$ none
#$ 7
# `[WK-11]` — a link may hold no target (`parent: weakref Node = None`): as a field's default, a
# local's value and a parameter's argument; an empty link and a link whose target ended both open
# to `else:`.

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None

fn show(cam: Camera):
    if cam.target exists as p:
        println(p.hp)
    else:
        println("gone")

fn hp_of(w: weakref Player) -> int:
    if w exists as p:
        return p.hp
    return 0

fn main():
    cam = Camera()
    show(cam)
    player = Player()
    cam.target = weakref player
    show(cam)
    player = Player(hp=5)
    show(cam)
    empty: weakref Player = None
    if empty exists as p:
        println(p.hp)
    else:
        println("none")
    seven = Player(hp=7)
    println(hp_of(weakref seven) + hp_of(None))
