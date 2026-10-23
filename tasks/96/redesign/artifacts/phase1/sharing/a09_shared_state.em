# A9: a player that the camera and the HUD both change (the Python way: two references to one object).

class Player:
    hp: int = 100
    inventory: Array[String] = []

class Camera:
    target: Player

class Hud:
    target: Player

fn main():
    p = Player()
    cam = Camera(p)
    hud = Hud(p)
    cam.target.hp -= 1
    hud.target.inventory.push("key".to_string())
    # Writing a Copy field while another field is iterated: never checked, accepted.
    for item in hud.target.inventory:
        cam.target.hp -= item.len()
    # Growing the very list being iterated, through another handle: compiles, panics at run time
    # (exclusivity violation, [EXC-1]); Python would loop forever or skip, Rust refuses at compile time.
    for item in hud.target.inventory:
        cam.target.inventory.push(item.clone())
