#$ test: compile-pass
#$ rules: WK-11, OWN-1
## design rules: R11, R1
## R11: the nearest allowed program: `weakref player` makes the link, and `player` stays `main`'s.

class Player:
    hp: int = 100

class Camera:
    target: weakref Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = weakref player
    println(player.hp)
