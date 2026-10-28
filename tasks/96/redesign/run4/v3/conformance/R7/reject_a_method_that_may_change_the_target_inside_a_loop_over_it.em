#$ test: compile-fail
#$ rules: R7, R8, R5
#$ help: cam.shake()
## R7, R8 (c): a call whose first line says `changes target` may change anything reached through the
## link, the player's inventory included, so it is refused inside a loop over that list. AN-48, point 6.

struct Player:
    hp: int
    inventory: Array[String]

struct Camera:
    target: sharedref mut Player

    fn shake() changes target:
        this.target.hp -= 1

struct Hud:
    target: sharedref mut Player

fn main():
    p = Player(hp=100, inventory=[String("key")])
    cam = Camera(target=sharedref mut p)
    hud = Hud(target=sharedref mut p)
    for item in hud.target.inventory:
        cam.shake()    #$ error[E3020]: `cam.shake()` says `changes target`, which may be the `Player` whose inventory this loop walks; write `cam.shake()` below the loop instead of in it
        println(item)
