#$ test: compile-fail
#$ rules: R7
#$ help: target: ref Player
## R7: `sharedref` is always followed by `mut` (E0114): a link that only reads is `ref T`, which any
## number of names may hold already. AN-52, point 6.

struct Player:
    hp: int

struct Camera:
    target: sharedref Player    #$ error[E0114]: a link that only reads may be held by any number of names: write `target: ref Player`; `sharedref` is for a link that may change its target: `target: sharedref mut Player`

fn main():
    p = Player(hp=100)
    cam = Camera(target=ref p)
    println(cam.target.hp)
