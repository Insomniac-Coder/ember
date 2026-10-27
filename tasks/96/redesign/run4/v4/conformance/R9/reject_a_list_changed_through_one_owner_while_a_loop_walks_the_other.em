#$ test: compile-fail
#$ rules: R9, R8
#$ note: `cam.target` and `hud.target` may be the same `Player`, and adding can move the list's elements while the loop reads them
#$ help: add after the loop: write `new_items: Array[String] = []` before the loop, `new_items.push(copy of item)` in place of line 25, and `cam.target.inventory.extend(new_items)` after the loop; or walk a list of your own: `for item in copy of hud.target.inventory:`
## R9: changing a shared value through any owner is safe because "a list does not change while a loop
## walks it, through whichever of its names" (R8 (c)); the camera and the HUD own one player
## (AN-48 (a), still refused); `main` says `changes Player`, a change through links (R5, verdict 64).

class Player:
    hp: int = 100
    inventory: Array[String] = []

class Camera:
    target: shares mut Player

class Hud:
    target: shares mut Player

fn main() changes Player:
    p = Player()
    cam = Camera(target=shares mut p)
    hud = Hud(target=shares mut p)
    hud.target.inventory.push(String("key"))
    for item in hud.target.inventory:
        cam.target.inventory.push(copy of item)    #$ error[E3020]: `cam.target.inventory.push(copy of item)` (line 25) may add to the list this loop walks (`hud.target.inventory`, line 24)
