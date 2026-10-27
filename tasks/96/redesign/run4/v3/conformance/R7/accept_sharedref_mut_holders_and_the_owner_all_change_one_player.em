#$ test: run-pass
#$ rules: R7, R8
#$ stdout: 106 106 1
## R7: `sharedref mut` may change, and several holders and the owner may name one object; on one thread
## any of them may change it at any time; a number changes freely during a walk of another list of
## the player (R8 (c)). AN-48's NEW program (b).

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
    p = Player(hp=100, inventory=[])
    cam = Camera(target=sharedref mut p)
    hud = Hud(target=sharedref mut p)
    cam.shake()
    hud.target.inventory.push(String("key"))
    for item in hud.target.inventory:
        cam.target.hp -= item.len()
    p.hp += 10
    println(p.hp, cam.target.hp, hud.target.inventory.len())
