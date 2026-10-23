# A9 with a struct Player: the same program as Rust's, two views holding `ref mut` to one value.

struct Player:
    hp: int
    inventory: Array[String]

struct Camera:
    target: ref mut Player

struct Hud:
    target: ref mut Player

fn main():
    p = Player(hp=100, inventory=[])
    cam = Camera(ref mut p)
    hud = Hud(ref mut p)
    cam.target.hp -= 1
    hud.target.inventory.push("key".to_string())
