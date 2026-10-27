#$ test: compile-pass
#$ rules: R11, R13
## R11: the nearest allowed program: the link is opened, and the opened name is used (AN-65's NEW).

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
