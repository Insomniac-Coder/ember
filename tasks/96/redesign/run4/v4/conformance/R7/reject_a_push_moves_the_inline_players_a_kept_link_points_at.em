#$ test: compile-fail
#$ rules: R7, R8
#$ help: w = Watcher(target=sharedref mut players[1])
## R7, R8 (b): a kept link is an address, not a position; into a list that holds its structs inline, a
## push may move the player it points at, so the push ends it (E3060 at a later use); verdict 81 keeps
## only a link to an object in its own block. `main` changes the player through the link, so it says
## `changes Player` (R5, verdict 64), and the line the message gives compiles.

struct Player:
    hp: int

struct Watcher:
    target: sharedref mut Player

fn main() changes Player:
    players = [Player(hp=10), Player(hp=10)]
    w = Watcher(target=sharedref mut players[1])
    players.push(Player(hp=10))
    w.target.hp -= 1    #$ error[E3060]: `players.push(Player(hp=10))` (line 18) may move the players `w.target` points into (line 17), and `w` is used on line 19; use `w` before line 18, or make it after the push: write `w = Watcher(target=sharedref mut players[1])` below line 18 in place of line 17
