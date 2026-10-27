#$ test: compile-fail
#$ rules: R7, R8
#$ help: w = Watcher(target=sharedref mut players[1])
## R7, R8 (b): a kept link is an address, not a position; into a list that holds its structs inline, a
## push may move the player it points at, so the push ends it (E3060 at a later use).

struct Player:
    hp: int

struct Watcher:
    target: sharedref mut Player

fn main():
    players = [Player(hp=10), Player(hp=10)]
    w = Watcher(target=sharedref mut players[1])
    players.push(Player(hp=10))
    w.target.hp -= 1    #$ error[E3060]: `players.push(Player(hp=10))` (line 16) may move the players `w.target` points into (line 15), and `w` is used on line 17; use `w` before line 16, or make it after the push: write `w = Watcher(target=sharedref mut players[1])` below line 16 in place of line 15
