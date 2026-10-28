#$ test: compile-fail
#$ rules: R9, R7, R8
#$ note: `cam.target` and `hud.target` may be the same `Player`, and adding can move the list's elements while the loop reads them
#$ help: add after the loop: write `new_items: Array[String] = []` before the loop, `new_items.push(copy of item)` in place of line 24, and `cam.target.inventory.extend(new_items)` after the loop; or walk a list of your own: `for item in copy of hud.target.inventory:`
## R9, R7: "a list in it may not change while a loop walks it, whichever holder reaches it" (R8 (c)):
## the HUD and the camera hold `sharedref mut` links to one player (AN-48 (b)); `main` says `changes Player` (verdict 64).

struct Player:
    hp: int
    inventory: Array[String]

struct Camera:
    target: sharedref mut Player

struct Hud:
    target: sharedref mut Player

fn main() changes Player:
    p = Player(hp=100, inventory=[])
    cam = Camera(target=sharedref mut p)
    hud = Hud(target=sharedref mut p)
    hud.target.inventory.push(String("key"))
    for item in hud.target.inventory:
        cam.target.inventory.push(copy of item)    #$ error[E3020]: `cam.target.inventory.push(copy of item)` (line 24) may add to the list this loop walks (`hud.target.inventory`, line 23)
