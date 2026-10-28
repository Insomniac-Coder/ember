#$ test: compile-fail
#$ rules: R7, R8
#$ help: target: shares mut Player
## R7, R8 (b): while a kept link is used, nothing it points at may be removed from the list that owns it
## (E3060 at a later use). AN-52, point 6 (form (b), with a `class Player` list).

class Player:
    hp: int = 10

struct Watcher:
    target: sharedref mut Player

fn main():
    players = [Player(), Player()]
    w = Watcher(target=sharedref mut players[1])
    players.remove(1)
    w.target.hp -= 1    #$ error[E3060]: `players.remove(1)` (line 16) would end a player that `w` (line 15) still points at on line 17; write `players.remove(1)` below line 17, after the last use of `w`; or let the watchers keep their players alive: write `target: shares mut Player` in `Watcher`, `players: Array[shares mut Player]`, and `Watcher(target=shares mut players[1])` where a watcher is made
