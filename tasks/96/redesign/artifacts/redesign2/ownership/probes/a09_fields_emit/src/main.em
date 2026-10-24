# A9: two handles to one Player. Which conflicts does `ember check` see today, and which are left
# to the run-time per-field access words ([EXC-19])?
class Player:
    hp: int = 100
    inventory: Array[String] = []
    log: Array[String] = []

class Camera:
    target: Player

class Hud:
    target: Player

class World:
    players: Array[Player] = []

    fn give_to_someone(self, owned item: String):
        self.players[0].inventory.push(item)

fn main():
    p = Player()
    w = World()
    w.players.push(p)
    cam = Camera(p)
    hud = Hud(p)
    hud.target.inventory.push("key")
    # 1. A Copy field written while a non-Copy field is iterated: never checked ([EXC-17]).
    for item in hud.target.inventory:
        cam.target.hp -= item.len()
    # 2. A different non-Copy field of the same object: the field words differ, so the run-time check passes.
    for item in hud.target.inventory:
        cam.target.log.push(item.clone())
    # 3. The same field through another path: accepted by the checker; [EXC-1] panics at run time.
    for item in hud.target.inventory:
        w.give_to_someone(item.clone())
