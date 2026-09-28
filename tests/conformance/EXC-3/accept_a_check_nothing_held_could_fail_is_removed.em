#$ test: run-pass
#$ rules: EXC-3, EXC-3a, EXC-19
#$ profiles: debug, release, shipping
#$ stdout: 6
# `[EXC-3]` — nothing in this program ever holds a player's list while other
# code runs, so no check on it can fail: the compiler removes them, and
# `[EXC-3a]` records each in the safety side table.

class Player:
    items: Array[int]

    fn init(mut self, n: int):
        self.items = []
        for i in 0..n:
            self.items.push(i)

fn main():
    players: Array[Player] = []
    for n in 1..4:
        players.push(Player(n))
    total = 0
    for p in players:
        total = total + len(p.items)
    println(total)
