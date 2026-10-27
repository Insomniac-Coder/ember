#$ test: run-pass
#$ rules: R9, R7
#$ stdout: 106 106
## R9: "any holder of a `sharedref mut` link to it may change it too"; R7: "On one thread any of them
## may change it at any time", the owner included. No count and no check: a plain pointer in each
## struct (AN-48 (b); section 2's `sharedref mut` row, "nothing").

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
    cam.shake()                                # 99
    hud.target.inventory.push(String("key"))
    for item in hud.target.inventory:
        cam.target.hp -= item.len()            # a number: 96
    p.hp += 10                                 # the owner may change it too: 106
    println(p.hp, hud.target.hp)
