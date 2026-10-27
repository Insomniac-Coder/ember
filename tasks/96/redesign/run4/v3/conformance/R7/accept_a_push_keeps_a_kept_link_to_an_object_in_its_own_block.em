#$ test: run-pass
#$ rules: R7, R8
#$ stdout: 9 3
## R7, R8 (b): a kept `sharedref mut` link into a list whose elements live in their own blocks (class
## objects) survives a push: growing moves only the list's pointers, never the players. AN-52's form (b),
## with a `class Player` list; see the matrix, R7-9 and for the owner, on what "into" covers.

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
