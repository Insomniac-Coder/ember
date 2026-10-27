#$ test: compile-fail
#$ rules: LT-45
#$ help: target: shares mut Player
## design rules: R7
## R7: a struct holding a `sharedref mut` link is a view, so it lives no longer than its target (E3063).
## AN-48, point 6.

struct Player:
    hp: int

struct Camera:
    target: sharedref mut Player

fn make_camera() -> Camera:
    p = Player(hp=100)
    cam = Camera(target=sharedref mut p)
    return cam    #$ error[E3063]: `cam` holds a link to `p`, which ends when `make_camera` returns, so `cam` may not outlive it (returned here); let the camera own a share that may change the player: write `target: shares mut Player` in `Camera`, and `cam = Camera(target=shares mut p)` in place of `cam = Camera(target=sharedref mut p)`

fn main():
    println(make_camera().target.hp)
