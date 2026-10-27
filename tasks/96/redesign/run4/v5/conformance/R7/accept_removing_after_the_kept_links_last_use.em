#$ test: run-pass
#$ rules: R7, R8
#$ stdout: 1 10
## R7, R8 (b): the first way out E3060's message gives: the removal comes after the last use of the
## watcher. AN-52, point 6.

class Player:
    hp: int = 10

struct Watcher:
    target: sharedref mut Player

fn main():
    players = [Player(), Player()]
    w = Watcher(target=sharedref mut players[1])
    w.target.hp -= 1
    players.remove(1)
    println(players.len(), players[0].hp)
