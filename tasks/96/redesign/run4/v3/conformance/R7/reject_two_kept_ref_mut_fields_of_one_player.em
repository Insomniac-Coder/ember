#$ test: compile-fail
#$ rules: R7, R8
#$ help: target: sharedref mut Player
## R7, R8 (f): a kept `ref mut` view is alone: while its holder is still used, the place is not lent to
## another `ref mut` field. AN-48's OLD struct form; the E3022 message of `V3/rules.md` section 3.

struct Player:
    hp: int

struct Camera:
    target: ref mut Player

struct Hud:
    target: ref mut Player

fn main():
    p = Player(hp=100)
    cam = Camera(target=ref mut p)
    hud = Hud(target=ref mut p)    #$ error[E3022]: `p` is lent to `cam.target`, a `ref mut` field, until its last use on line 20, and line 19 lends it to `hud.target` too; if both may change the player, write `target: sharedref mut Player` in `Camera` and in `Hud`
    cam.target.hp -= 1
    hud.target.hp -= 1
