#$ test: compile-fail
#$ rules: BRW-1
#$ help: cam.target.hp
## design rules: R7, R8
## R7, R8 (f): while the holder of a kept `ref mut` view is still used, the place is not read through
## another name, its owner included (verdict 2).

struct Player:
    hp: int

struct Camera:
    target: ref mut Player

fn main():
    p = Player(hp=100)
    cam = Camera(target=ref mut p)
    x = p.hp    #$ error[E3021]: `cam.target` is the only name for `p` until line 17, and line 16 reads `p.hp`; read it through `cam.target`: `cam.target.hp`
    cam.target.hp -= x
