# A15: each scoped task changes its own element of a list of structs (disjoint writes).
import std.thread

struct Player:
    hp: int

fn hurt_all(mut players: Array[Player]):
    with scope = thread.scope():
        for p in players.iter_mut():
            scope.spawn(fn(): p.hp -= 1)

fn main():
    ps = [Player(hp=10), Player(hp=20)]
    hurt_all(ps)
