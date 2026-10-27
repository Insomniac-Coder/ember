#$ test: compile-fail
#$ rules: R7, R8
#$ help: new_items: Array[String] = []
## R7, R8 (c): a list in a `sharedref mut` target may not change while a `for` loop walks it, whichever
## holder reaches it. AN-48, point 6 (its message for form (a), here for form (b), as one line).

struct Player:
    hp: int
    inventory: Array[String]

struct Camera:
    target: sharedref mut Player

struct Hud:
    target: sharedref mut Player

fn main():
    p = Player(hp=100, inventory=[String("key")])
    cam = Camera(target=sharedref mut p)
    hud = Hud(target=sharedref mut p)
    for item in hud.target.inventory:
        cam.target.inventory.push(copy of item)    #$ error[E3020]: `cam.target.inventory.push(copy of item)` (line 22) may add to the list this loop walks (`hud.target.inventory`, line 21): `cam.target` and `hud.target` may be the same `Player`, and adding can move the list's elements while the loop reads them; add after the loop: write `new_items: Array[String] = []` before the loop, `new_items.push(copy of item)` in place of line 22, and `cam.target.inventory.extend(new_items)` after the loop; or walk a list of your own: `for item in copy of hud.target.inventory:`
