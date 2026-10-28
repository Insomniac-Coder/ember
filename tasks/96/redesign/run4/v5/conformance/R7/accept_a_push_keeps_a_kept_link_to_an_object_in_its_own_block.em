#$ test: run-pass
#$ rules: LT-45, BRW-16
#$ stdout: 9 3
## design rules: R7, R8
## R7, R8 (b), verdict 81: a kept `sharedref mut` link to an object that lives in its own block (a class
## object) survives a push to the list that owns it: growing moves only the list's pointers, never the
## players. Verdict 81's own example.

class Player:
    hp: int = 10

struct Watcher:
    target: sharedref mut Player

fn main():
    players = [Player(), Player()]
    w = Watcher(target=sharedref mut players[1])
    players.push(Player())
    w.target.hp -= 1
    println(players[1].hp, players.len())
