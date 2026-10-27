#$ test: run-pass
#$ rules: LT-45, BRW-16
#$ stdout: 98
## design rules: R7, R8
## R7: a kept `ref mut` view is alone only while its holder is still used; after the holder's last use
## the owner reads and changes the player again.

struct Player:
    hp: int

struct Camera:
    target: ref mut Player

fn main():
    p = Player(hp=100)
    cam = Camera(target=ref mut p)
    cam.target.hp -= 1
    p.hp -= 1
    println(p.hp)
