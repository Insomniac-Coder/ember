# The loop over `g.players` calls the other package's `note`, which touches only `log` and `frame`.
from gamelib import Game, Player

fn tick(mut g: Game):
    for p in g.players.iter_mut():
        p.hp -= 1
        if p.hp <= 0:
            g.note(f"{p.name} died")

fn main():
    g = Game(players=[Player(name="orc", hp=1)], log=[])
    tick(g)
    println(g.log)
