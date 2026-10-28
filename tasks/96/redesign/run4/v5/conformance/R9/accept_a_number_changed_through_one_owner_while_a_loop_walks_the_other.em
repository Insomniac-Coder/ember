#$ test: compile-pass
#$ rules: RC-1, BRW-16, FN-4
## design rules: R9, R8, R5
## R9, R8 (c): while a loop walks the player's inventory through one owner, the other owner may change
## a number of the player and another list of it; only the walked list may not change (AN-48 (a)).

class Player:
    hp: int = 100
    inventory: Array[String] = []
    log: Array[String] = []

class Camera:
    target: shares mut Player

class Hud:
    target: shares mut Player

fn main():
    p = Player()
    cam = Camera(target=shares mut p)
    hud = Hud(target=shares mut p)
    hud.target.inventory.push(String("key"))
    for item in hud.target.inventory:
        cam.target.hp -= item.len()            # a number: accepted
    for item in hud.target.inventory:
        cam.target.log.push(copy of item)      # another list of the player: accepted
