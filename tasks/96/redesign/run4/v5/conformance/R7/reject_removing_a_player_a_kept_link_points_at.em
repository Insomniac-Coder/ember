#$ test: compile-fail
#$ rules: R7, R8
#$ help: target: shares mut Player
## R7, R8 (b): while a kept link is used, nothing it points at may be removed from the list that owns it
## (E3060 at a later use). AN-52, point 6 (form (b), with a `class Player` list), its second way out
## written as whole lines (R18).

class Player:
    hp: int = 10

struct Watcher:
    target: sharedref mut Player

fn main():
    players = [Player(), Player()]
    w = Watcher(target=sharedref mut players[1])
    players.remove(1)
    w.target.hp -= 1    #$ error[E3060]: `players.remove(1)` (line 17) may end the player `w.target` points at (`players[1]`, line 16), and `w` is used on line 18; write `players.remove(1)` below line 18, after `w`'s last use; or let the watcher keep its player alive: write `target: shares mut Player` in `Watcher`, `players = [shares mut Player(), shares mut Player()]` in place of line 15 and `w = Watcher(target=shares mut players[1])` in place of line 16
