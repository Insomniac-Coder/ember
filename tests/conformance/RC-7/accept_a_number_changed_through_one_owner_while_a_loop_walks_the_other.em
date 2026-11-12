#$ test: compile-pass
#$ rules: RC-7, RC-1
## Design test `accept_a_number_changed_through_one_owner_while_a_loop_walks_the_other.em` of `tasks/96/redesign/run4/v3/conformance/`; where it says so, `owns` is written at an `owns` parameter (`[FN-2a]`) and a `changes` list on a method's first line is left out.
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
    hud.target.inventory.push(owns String("key"))
    for item in hud.target.inventory:
        cam.target.hp -= item.len()            # a number: accepted
    for item in hud.target.inventory:
        cam.target.log.push(owns copy of item)      # another list of the player: accepted
