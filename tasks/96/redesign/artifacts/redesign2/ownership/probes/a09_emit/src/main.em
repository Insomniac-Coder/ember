# The shared-object workload's inner step in today's Ember, to read what the compiler emits.
class Player:
    hp: i32 = 1000
    x: i32 = 0
    inventory: Array[i32] = []

struct Watcher:
    target: Player

fn step(ws: Array[Watcher], frame: i32) -> u64:
    acc: u64 = 0
    for i in range(ws.len()):
        t = ws[i].target
        t.hp -= 1
        acc += (t.x as u64) * ((t.inventory.len() + 1) as u64)
        if (i & 7) == (frame & 7) as int:
            t.inventory.push(frame)
            if t.inventory.len() > 8:
                for v in t.inventory:
                    acc += v as u64
                t.inventory.clear()
    return acc

fn retarget(mut ws: Array[Watcher], players: Array[Player], i: int, r: int):
    ws[i].target = players[r]

fn main():
    players: Array[Player] = [Player(), Player()]
    ws: Array[Watcher] = [Watcher(target=players[0])]
    println(step(ws, 0))
    retarget(ws, players, 0, 1)
