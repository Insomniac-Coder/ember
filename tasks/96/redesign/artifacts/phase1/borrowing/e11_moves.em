# A11: use after move, partial move, move in a loop, and Python-style aliasing.
struct Msg:
    to: String
    body: String

fn send(owned m: Msg):
    pass

fn log(m: Msg):
    pass

fn notify(owned m: Msg):
    send(m)
    log(m)

struct Player:
    name: String
    inventory: Array[String]

fn keep(owned p: Player):
    pass

fn rename(owned p: Player):
    n = p.name
    println(n)
    keep(p)

fn spawn_all(owned template: Player, n: int) -> Array[Player]:
    v: Array[Player] = []
    for _ in 0..n:
        v.push(template)
    return v

fn alias():
    items: Array[int] = [1, 2]
    snapshot = items
    items.push(3)
    println(snapshot)

fn main():
    alias()
