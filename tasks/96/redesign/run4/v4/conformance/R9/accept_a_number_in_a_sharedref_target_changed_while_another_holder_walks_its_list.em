#$ test: compile-pass
#$ rules: R9, R7, R8, R5
## R9, R7: the nearest allowed program: while one holder's loop walks the player's list, another holder
## changes a number of the player (AN-48 (b)). A `sharedref mut` field is a link (rules section 0), so
## `main`, which changes the player through it, says `changes Player` (R5, verdict 64).

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
        cam.target.hp -= item.len()
